shared_config = {
  database = {
    engine = "mysql"
    port   = 3306
  }
  
  logging = {
    level = "info"
    format = "json"
  }
}

app_settings = {
  timeout = 30
}
