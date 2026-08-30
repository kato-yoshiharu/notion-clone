provider "aws" {
  region  = var.region
  profile = var.aws_profile

  default_tags {
    tags = {
      Project   = "notion-clone"
      ManagedBy = "terraform"
    }
  }
}
