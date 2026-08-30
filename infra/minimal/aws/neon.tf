# stateを分けているのは、applyの範囲をこちらに限定してDBの再作成事故を避けるため。
# 参照先はneon側のbackendと同じバケット・キーを指す。
data "terraform_remote_state" "neon" {
  backend = "s3"

  config = {
    bucket = "kato-yoshiharu-notion-clone-tfstate"
    key    = "neon/terraform.tfstate"
    region = "ap-northeast-1"
  }
}

locals {
  neon_connection_uri = data.terraform_remote_state.neon.outputs.connection_uri

  # Neonが返すURIに sslmode が無い場合があるため補う。既存のクエリを壊さないよう区切りを出し分ける。
  has_sslmode = strcontains(local.neon_connection_uri, "sslmode=")
  has_query   = strcontains(local.neon_connection_uri, "?")

  database_url = (
    local.has_sslmode ? local.neon_connection_uri :
    local.has_query ? "${local.neon_connection_uri}&sslmode=require" :
    "${local.neon_connection_uri}?sslmode=require"
  )
}
