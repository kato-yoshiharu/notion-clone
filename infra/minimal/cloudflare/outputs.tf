output "main_token_id" {
  description = "ID of the Terraform-managed main API token."
  value       = cloudflare_account_token.main.id
}

output "main_token_value" {
  description = "Secret value of the main API token. Shown only right after creation."
  value       = cloudflare_account_token.main.value
  sensitive   = true
}
