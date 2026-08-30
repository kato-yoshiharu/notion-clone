locals {
  function_name = "notion-clone-backend"

  # `cargo make lambda-build` の成果物。
  bootstrap_path = "${path.module}/../../../target/lambda/backend/bootstrap"
}

# provided.al2023 はzip内の `bootstrap` をエントリポイントとして扱う。
data "archive_file" "backend" {
  type        = "zip"
  output_path = "${path.module}/.terraform/backend.zip"
  source_file = local.bootstrap_path
}

data "aws_iam_policy_document" "lambda_assume_role" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "backend" {
  name               = "${local.function_name}-role"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume_role.json
}

resource "aws_iam_role_policy_attachment" "backend_logs" {
  role       = aws_iam_role.backend.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# Lambdaに自動生成させると保持期間が無期限になるため、先に作って明示する。
resource "aws_cloudwatch_log_group" "backend" {
  name              = "/aws/lambda/${local.function_name}"
  retention_in_days = var.log_retention_days
}

resource "aws_lambda_function" "backend" {
  function_name = local.function_name
  role          = aws_iam_role.backend.arn

  # arm64(Graviton)はx86_64より約20%安い。
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  handler       = "bootstrap"

  filename         = data.archive_file.backend.output_path
  source_code_hash = data.archive_file.backend.output_base64sha256

  memory_size = 512
  timeout     = 30

  environment {
    variables = {
      DATABASE_URL = local.database_url
      # 1コンテナが同時に1リクエストしか処理しないため、プールは1本で足りる。
      DATABASE_MAX_CONNECTIONS = "1"
      ORIGIN_SHARED_SECRET     = var.origin_shared_secret
      RUST_LOG                 = "backend=info"
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.backend_logs,
    aws_cloudwatch_log_group.backend,
  ]
}

# API Gatewayは従量課金のため、無料のFunction URLをオリジンにする。
# AWS_IAM認証はCloudflare側でSigV4署名が必要になり現実的でないので、NONEとしてORIGIN_SHARED_SECRETをアプリ側で検証する。
resource "aws_lambda_function_url" "backend" {
  function_name      = aws_lambda_function.backend.function_name
  authorization_type = "NONE"
}

# 2025年10月以降、Function URLの呼び出しには lambda:InvokeFunctionUrl だけでなく lambda:InvokeFunction の許可も必要。
# aws_lambda_function_urlは前者しか作らないため、これが無いとauthorization_type = "NONE"でも403 Forbiddenになる。
# invoked_via_function_urlを付けて、直接Invokeされる経路は許可しない。
resource "aws_lambda_permission" "function_url_invoke" {
  statement_id             = "FunctionURLInvokeAllowPublicAccess"
  action                   = "lambda:InvokeFunction"
  function_name            = aws_lambda_function.backend.function_name
  principal                = "*"
  invoked_via_function_url = true
}
