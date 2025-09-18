vpc_cidr = "10.0.0.0/16"
availability_zones = ["us-west-2a", "us-west-2b"]

subnets = {
  public = {
    cidr = "10.0.1.0/24"
    type = "public"
  }
  private = {
    cidr = "10.0.2.0/24"
    type = "private"
  }
}
