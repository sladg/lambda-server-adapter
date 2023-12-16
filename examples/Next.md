# NextJS

There are multiple parts to making this work. Firstly, you need to make sure `output: 'standalone'` is set. In case you are using ISR, you should also set `experimental.isrMemoryCacheSize = 0` to avoid cache being stored in memory.

Keep in mind that Lambda's Read-Only FS does not allow for efficient caching, this is under contruction for now (affects ISR, image optimization).

## Terraform

### With streaming

Use `invoke_mode = "RESPONSE_STREAM"` with `USE_STREAM = true` in environment variables.

### Without streaming

Use `invoke_mode = "BUFFERED"` with `USE_STREAM = false` or omit env var completely.

### Definition

```hcl
module "server_function" {
  source  = "terraform-aws-modules/lambda/aws"
  version = "6.4.0"

  lambda_at_edge = false
  snap_start     = false

  function_name              = "nextjs-function"
  handler                    = "./server.js"
  runtime                    = "nodejs20.x"
  architectures              = ["arm64"]
  memory_size                = 512
  ephemeral_storage_size     = 512
  timeout                    = 30
  publish                    = true
  create_lambda_function_url = true
  create_package             = true
  store_on_s3                = false

  source_path = [
    {
      path             = ".next/standalone"
      npm_requirements = false
      patterns = [
        # Exclude binaries for other platforms. This will exclude Prisma's binary
        "!.*/.*darwin.*\\.node"
      ]
    },
    {
      path          = "public"
      prefix_in_zip  = "public"
    },
    {
      path          = ".next/static"
      prefix_in_zip  = ".next/static"
    }
  ]

  artifacts_dir = ".terraform"

  invoke_mode = "RESPONSE_STREAM"

  attach_cloudwatch_logs_policy     = true
  cloudwatch_logs_retention_in_days = 1

  environment_variables = {
    AWS_LAMBDA_EXEC_WRAPPER = "/opt/lambda-adapter/bootstrap" // Neccessary for Runtime API
    SERVER_URL              = "http://localhost:3000/api/health"  // Adapter waits for this to be active
    USE_STREAM              = true // Use stream mode for faster responses
    NODE_ENV                = "production"
  }

  layers = [
    "arn:aws:lambda: ....... insert your arn ...... " // Network Adapter
  ]
}
```

## Manually

- `next build` (in standalone mode),
- `cp -r ./public .next/standalone/public`,
- `cp -r .next/static/ .next/standalone/.next/static`,
- zip the folder and upload it as Lambda code,
- add `lambda-server-adapter` as a layer,
- set handler as path to `server.js` file (aka `/var/task/server.js`),
- set `SERVER_URL=http://localhost:3000` as environment variable (best is to use `/api/health` route),
- set `AWS_LAMBDA_EXEC_WRAPPER = "/opt/lambda-adapter/bootstrap"` to use the wrapper.
