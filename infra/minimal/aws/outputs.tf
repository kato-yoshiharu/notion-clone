output "function_name" {
  description = "Name of the backend Lambda function."
  value       = aws_lambda_function.backend.function_name
}

output "function_url" {
  description = "Lambda Function URL. Set this as the origin of the Cloudflare route."
  value       = aws_lambda_function_url.backend.function_url
}
