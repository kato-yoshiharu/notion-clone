output "state_bucket_name" {
  description = "S3 bucket for terraform state. Must match the backend blocks of neon / aws / cloudflare."
  value       = aws_s3_bucket.state.id
}

output "github_actions_role_arn" {
  description = "Role ARN for aws-actions/configure-aws-credentials. Store it in GitHub Secrets."
  value       = aws_iam_role.github_actions.arn
}
