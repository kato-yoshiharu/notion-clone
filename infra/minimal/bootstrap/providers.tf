provider "aws" {
  region = var.region

  default_tags {
    tags = {
      Project   = "notion-clone"
      ManagedBy = "terraform"
    }
  }
}
