output "function_url" {
  description = "Lambda Function URL. Set this as the origin of the Cloudflare route."
  value       = aws_lambda_function_url.backend.function_url
}
