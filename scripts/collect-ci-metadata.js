const { appendFileSync } = require("node:fs")
const { execFileSync } = require("node:child_process")

appendFileSync(
  process.env.GITHUB_OUTPUT,
  `has_library=${JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], { encoding: "utf8" }),
  ).packages.some((pkg) => pkg.targets.some((target) => target.kind.includes("lib")))}\n`,
)
