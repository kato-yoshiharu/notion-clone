output "connection_uri" {
  description = "Direct connection URI, credentials included."
  value       = neon_project.main.connection_uri
  sensitive   = true
}
