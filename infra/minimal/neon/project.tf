resource "neon_project" "main" {
  name                      = var.project_name
  org_id                    = var.org_id
  region_id                 = var.region_id
  pg_version                = var.pg_version
  history_retention_seconds = var.history_retention_seconds

  branch {
    name          = var.branch_name
    database_name = var.database_name
    role_name     = var.role_name
  }

  default_endpoint_settings {
    # 低トラフィック前提のため最小構成。無操作時は自動でサスペンドされる。
    autoscaling_limit_min_cu = 0.25
    autoscaling_limit_max_cu = 1.0
  }

  lifecycle {
    prevent_destroy = true
  }
}
