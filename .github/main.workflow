workflow "New workflow" {
  on = "push"
  resolves = ["Build"]
}

action "Build" {
  uses = "docker://rust:latest"
  runs = "/bin/cargo"
  args = "build --verbose"
}
