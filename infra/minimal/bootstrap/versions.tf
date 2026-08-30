terraform {
  required_version = ">= 1.14"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }

  # 鶏卵問題を避けるため、bootstrapのstateだけはローカルに置き、手動applyのみとする。
  backend "local" {}
}
