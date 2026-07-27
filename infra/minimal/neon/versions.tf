terraform {
  required_version = ">= 1.14"

  required_providers {
    neon = {
      source  = "kislerdm/neon"
      version = "0.14.0"
    }
  }
}

# 認証は環境変数 NEON_API_KEY から読む。
provider "neon" {}
