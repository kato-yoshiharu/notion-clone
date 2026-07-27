variable "org_id" {
  description = "Neon organization ID."
  type        = string
}

variable "project_name" {
  description = "Neon project name."
  type        = string
  default     = "notion-clone"
}

variable "region_id" {
  description = "Neon deployment region. No Tokyo region, so Singapore is closest."
  type        = string
  default     = "aws-ap-southeast-1"
}

variable "pg_version" {
  description = "Postgres major version."
  type        = number
  default     = 18
}

variable "branch_name" {
  description = "Name of the default branch."
  type        = string
  default     = "main"
}

variable "database_name" {
  description = "Name of the default database."
  type        = string
  default     = "notion_clone"
}

variable "role_name" {
  description = "Name of the default database role."
  type        = string
  default     = "notion_clone_owner"
}

variable "history_retention_seconds" {
  description = "Point-in-time restore window. Free plan default is 6 hours."
  type        = number
  default     = 21600
}
