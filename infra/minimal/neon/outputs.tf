output "project_id" {
  description = "Neon project ID."
  value       = neon_project.main.id
}

output "database_host" {
  description = "Default database host."
  value       = neon_project.main.database_host
}

output "connection_uri" {
  description = "Direct connection URI, credentials included."
  value       = neon_project.main.connection_uri
  sensitive   = true
}

output "connection_uri_pooler" {
  description = "Connection URI via the PgBouncer pooler. Unused: sqlx needs prepared statements."
  value       = neon_project.main.connection_uri_pooler
  sensitive   = true
}
