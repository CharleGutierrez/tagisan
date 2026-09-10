# Deployment Automation with Bun Shell

## SSH Deployment

```typescript
import { $ } from "bun";

interface DeployConfig {
  host: string;
  user: string;
  remotePath: string;
  localPath: string;
}

async function deployViaSsh(config: DeployConfig) {
  const { host, user, remotePath, localPath } = config;
  const remote = `${user}@${host}`;

  console.log(`Deploying to ${remote}:${remotePath}...`);

  // Build locally first
  await $`bun run build`;

  // Create remote directory
  await $`ssh ${remote} "mkdir -p ${remotePath}"`;

  // Sync files
  await $`rsync -avz --delete ${localPath}/ ${remote}:${remotePath}/`;

  // Restart service (example with PM2)
  await $`ssh ${remote} "cd ${remotePath} && pm2 restart app"`;

  console.log("Deployment complete!");
}
```

## Docker Deployment

```typescript
import { $ } from "bun";

async function dockerDeploy(imageName: string, containerName: string) {
  // Build image
  console.log("Building Docker image...");
  await $`docker build -t ${imageName} .`;

  // Stop existing container
  console.log("Stopping existing container...");
  await $`docker stop ${containerName}`.nothrow();
  await $`docker rm ${containerName}`.nothrow();

  // Run new container
  console.log("Starting new container...");
  await $`docker run -d --name ${containerName} -p 3000:3000 ${imageName}`;

  // Health check
  await $`sleep 5`;
  const health = await $`docker inspect --format='{{.State.Health.Status}}' ${containerName}`.nothrow().text();
  console.log(`Container health: ${health.trim() || 'running'}`);
}

async function dockerComposeUpgrade() {
  // Pull latest images
  await $`docker compose pull`;

  // Rebuild and restart
  await $`docker compose up -d --build`;

  // Clean old images
  await $`docker image prune -f`;
}
```

## Vercel/Netlify CLI Deploy

```typescript
import { $ } from "bun";

async function deployVercel(production = false) {
  // Build first
  await $`bun run build`;

  // Deploy
  const prodFlag = production ? '--prod' : '';
  const output = await $`vercel ${prodFlag}`.text();

  // Extract URL from output
  const urlMatch = output.match(/https:\/\/[\w.-]+\.vercel\.app/);
  return urlMatch?.[0];
}

async function deployNetlify(production = false) {
  await $`bun run build`;

  const prodFlag = production ? '--prod' : '';
  await $`netlify deploy ${prodFlag} --dir=dist`;
}
```

## AWS S3 Static Site Deploy

```typescript
import { $ } from "bun";

async function deployToS3(bucket: string, distribution?: string) {
  // Build
  await $`bun run build`;

  // Sync to S3
  console.log(`Syncing to s3://${bucket}...`);
  await $`aws s3 sync dist/ s3://${bucket} --delete`;

  // Invalidate CloudFront cache if distribution ID provided
  if (distribution) {
    console.log("Invalidating CloudFront cache...");
    await $`aws cloudfront create-invalidation --distribution-id ${distribution} --paths "/*"`;
  }

  console.log(`Deployed to https://${bucket}.s3.amazonaws.com`);
}
```

## Blue-Green Deployment

```typescript
import { $ } from "bun";

async function blueGreenDeploy(serviceName: string) {
  // Determine current active (blue or green)
  const current = (await $`cat /tmp/${serviceName}-active`.nothrow().text()).trim() || 'blue';
  const next = current === 'blue' ? 'green' : 'blue';

  console.log(`Current: ${current}, Deploying to: ${next}`);

  // Deploy to inactive environment
  await $`bun run build`;
  await $`docker compose -f docker-compose.${next}.yml up -d --build`;

  // Health check
  const healthEndpoint = next === 'blue' ? 'http://localhost:3001/health' : 'http://localhost:3002/health';
  const { exitCode } = await $`curl -sf ${healthEndpoint}`.nothrow();

  if (exitCode !== 0) {
    console.error("Health check failed! Rolling back...");
    await $`docker compose -f docker-compose.${next}.yml down`;
    process.exit(1);
  }

  // Switch traffic (update nginx/haproxy config)
  await $`ln -sf /etc/nginx/sites-available/${next} /etc/nginx/sites-enabled/app`;
  await $`nginx -t && nginx -s reload`;

  // Update active marker
  await $`echo ${next} > /tmp/${serviceName}-active`;

  console.log(`Traffic switched to ${next}`);
}
```

## Kubernetes Deployment

```typescript
import { $ } from "bun";

async function k8sDeploy(deployment: string, image: string, tag: string) {
  const fullImage = `${image}:${tag}`;

  console.log(`Deploying ${deployment} with image ${fullImage}...`);

  // Build and push image
  await $`docker build -t ${fullImage} .`;
  await $`docker push ${fullImage}`;

  // Update deployment
  await $`kubectl set image deployment/${deployment} app=${fullImage}`;

  // Wait for rollout
  await $`kubectl rollout status deployment/${deployment}`;

  // Get pods
  await $`kubectl get pods -l app=${deployment}`;
}

async function k8sRollback(deployment: string) {
  console.log(`Rolling back ${deployment}...`);
  await $`kubectl rollout undo deployment/${deployment}`;
  await $`kubectl rollout status deployment/${deployment}`;
}
```

## Database Migration on Deploy

```typescript
import { $ } from "bun";

async function deployWithMigrations(env: 'staging' | 'production') {
  // Set environment
  $.env({ NODE_ENV: env });

  console.log(`Deploying to ${env}...`);

  // Build
  await $`bun run build`;

  // Run migrations
  console.log("Running database migrations...");
  const { exitCode } = await $`bun run migrate`.nothrow();

  if (exitCode !== 0) {
    console.error("Migration failed! Aborting deployment.");
    process.exit(1);
  }

  // Deploy
  if (env === 'production') {
    await $`vercel --prod`;
  } else {
    await $`vercel`;
  }

  console.log(`Deployed to ${env}!`);
}
```

## Deploy Status and Notifications

```typescript
import { $ } from "bun";

interface DeployStatus {
  success: boolean;
  url?: string;
  duration: number;
  commit: string;
}

async function deployWithNotification(): Promise<DeployStatus> {
  const start = Date.now();
  const commit = (await $`git rev-parse --short HEAD`.text()).trim();

  try {
    // Deploy
    await $`bun run build`;
    const output = await $`vercel --prod`.text();

    const urlMatch = output.match(/https:\/\/[\w.-]+\.vercel\.app/);
    const duration = (Date.now() - start) / 1000;

    // Success notification (webhook example)
    const payload = JSON.stringify({
      text: `Deployed ${commit} in ${duration}s`,
      url: urlMatch?.[0]
    });
    await $`curl -X POST -H "Content-Type: application/json" -d ${payload} ${process.env.SLACK_WEBHOOK}`.nothrow();

    return {
      success: true,
      url: urlMatch?.[0],
      duration,
      commit
    };

  } catch (err) {
    const duration = (Date.now() - start) / 1000;

    // Failure notification
    await $`curl -X POST -H "Content-Type: application/json" -d '{"text":"Deploy failed!"}' ${process.env.SLACK_WEBHOOK}`.nothrow();

    return {
      success: false,
      duration,
      commit
    };
  }
}
```

## Automated Rollback

```typescript
import { $ } from "bun";

async function deployWithAutoRollback(healthEndpoint: string) {
  // Save current state
  const prevCommit = (await $`git rev-parse HEAD~1`.text()).trim();

  try {
    // Deploy
    await $`bun run build`;
    await $`vercel --prod`;

    // Wait for deployment to be live
    await $`sleep 30`;

    // Health check (3 retries)
    for (let i = 0; i < 3; i++) {
      const { exitCode } = await $`curl -sf ${healthEndpoint}`.nothrow();
      if (exitCode === 0) {
        console.log("Health check passed!");
        return true;
      }
      await $`sleep 10`;
    }

    throw new Error("Health check failed after 3 retries");

  } catch (err) {
    console.error("Deployment failed, rolling back...");

    // Rollback to previous commit
    await $`git checkout ${prevCommit}`;
    await $`bun run build`;
    await $`vercel --prod`;

    throw err;
  }
}
```
