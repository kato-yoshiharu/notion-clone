variable "aws_profile" {
  description = "AWS profile for the notion-clone member account."
  type        = string
  default     = "notion-clone"
}

variable "region" {
  description = "AWS region."
  type        = string
  default     = "ap-northeast-1"
}

variable "cors_allowed_origin" {
  description = "Frontend origin allowed by the backend's CORS layer."
  type        = string
}

variable "origin_shared_secret" {
  description = "Shared secret sent by Cloudflare in the X-Origin-Secret header."
  type        = string
  sensitive   = true
}

variable "log_retention_days" {
  description = "CloudWatch Logs retention. Kept short to stay inside the free tier."
  type        = number
  default     = 14
}
