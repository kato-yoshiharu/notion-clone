variable "region" {
  description = "AWS region."
  type        = string
  default     = "ap-northeast-1"
}

variable "state_bucket_name" {
  description = "S3 bucket that holds the neon / aws / cloudflare states. Must match the backend blocks."
  type        = string
  default     = "kato-yoshiharu-notion-clone-tfstate"
}

variable "github_repository" {
  description = "GitHub repository allowed to assume the deploy role, in owner/name form."
  type        = string
  default     = "kato-yoshiharu/notion-clone"
}

variable "github_subjects" {
  description = "OIDC subjects allowed to assume the deploy role, relative to the repository."
  type        = list(string)
  default     = ["ref:refs/heads/main", "pull_request"]
}

variable "deploy_role_name" {
  description = "Name of the IAM role assumed by GitHub Actions."
  type        = string
  default     = "notion-clone-github-actions-deploy"
}
