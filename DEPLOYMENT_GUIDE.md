# Production Deployment Guide

Complete guide for deploying Passwordless Auth Server to production.

## 📋 **Pre-Deployment Checklist**

### Security
- [ ] Change `jwt_secret` to a strong random value (min 32 characters)
- [ ] Update all SMTP credentials
- [ ] Configure CORS allowed origins (disable `cors_allow_all`)
- [ ] Set appropriate rate limits for your use case
- [ ] Configure webhook secret if using webhooks
- [ ] Review and adjust security headers
- [ ] Enable HTTPS/TLS (via reverse proxy)

### Configuration
- [ ] Set production database path
- [ ] Configure email templates
- [ ] Set appropriate token expiry times
- [ ] Configure WebAuthn RP ID to match your domain
- [ ] Set server host/port
- [ ] Configure log level (info or warn for production)

### Infrastructure
- [ ] Database backup strategy
- [ ] Monitoring setup (Prometheus/Grafana)
- [ ] Log aggregation (ELK, Loki, etc.)
- [ ] Health check monitoring
- [ ] Alerting rules

## 🐳 **Docker Deployment**

### Build Image
```bash
docker build -t passwordless-auth:0.2.0 .
```

### Run with Docker
```bash
docker run -d \
  --name passwordless-auth \
  -p 3000:3000 \
  -e JWT_SECRET="your-production-secret-key-min-32-chars" \
  -e DATABASE_PATH="/data/auth.db" \
  -e SMTP_HOST="smtp.gmail.com" \
  -e SMTP_PORT=587 \
  -e SMTP_USERNAME="your-email@gmail.com" \
  -e SMTP_PASSWORD="your-app-password" \
  -e EMAIL_FROM="noreply@yourapp.com" \
  -e WEBAUTHN_RP_ID="yourapp.com" \
  -e WEBAUTHN_ORIGIN="https://yourapp.com" \
  -e CORS_ALLOWED_ORIGINS="https://yourapp.com,https://www.yourapp.com" \
  -e LOG_LEVEL="info" \
  -v /path/to/data:/data \
  -v /path/to/config.toml:/app/config.toml:ro \
  passwordless-auth:0.2.0
```

### Docker Compose
```yaml
version: '3.8'

services:
  auth:
    image: passwordless-auth:0.2.0
    ports:
      - "3000:3000"
    environment:
      JWT_SECRET: ${JWT_SECRET}
      DATABASE_PATH: /data/auth.db
      SMTP_HOST: ${SMTP_HOST}
      SMTP_PORT: ${SMTP_PORT}
      SMTP_USERNAME: ${SMTP_USERNAME}
      SMTP_PASSWORD: ${SMTP_PASSWORD}
      EMAIL_FROM: ${EMAIL_FROM}
      WEBAUTHN_RP_ID: ${WEBAUTHN_RP_ID}
      WEBAUTHN_ORIGIN: ${WEBAUTHN_ORIGIN}
      CORS_ALLOWED_ORIGINS: ${CORS_ALLOWED_ORIGINS}
      LOG_LEVEL: info
    volumes:
      - ./data:/data
      - ./config.toml:/app/config.toml:ro
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    restart: unless-stopped

volumes:
  prometheus-data:
  grafana-data:
```

## ☸️ **Kubernetes Deployment**

### Deployment Manifest
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: passwordless-auth
  labels:
    app: passwordless-auth
spec:
  replicas: 3
  selector:
    matchLabels:
      app: passwordless-auth
  template:
    metadata:
      labels:
        app: passwordless-auth
    spec:
      containers:
      - name: auth
        image: passwordless-auth:0.2.0
        ports:
        - containerPort: 3000
          name: http
        env:
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: auth-secrets
              key: jwt-secret
        - name: SMTP_PASSWORD
          valueFrom:
            secretKeyRef:
              name: auth-secrets
              key: smtp-password
        - name: DATABASE_PATH
          value: "/data/auth.db"
        - name: LOG_LEVEL
          value: "info"
        - name: WEBAUTHN_RP_ID
          value: "yourapp.com"
        - name: WEBAUTHN_ORIGIN
          value: "https://yourapp.com"
        volumeMounts:
        - name: data
          mountPath: /data
        - name: config
          mountPath: /app/config.toml
          subPath: config.toml
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /liveness
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /readiness
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: auth-data-pvc
      - name: config
        configMap:
          name: auth-config
---
apiVersion: v1
kind: Service
metadata:
  name: passwordless-auth
spec:
  selector:
    app: passwordless-auth
  ports:
  - port: 80
    targetPort: 3000
    name: http
  type: ClusterIP
---
apiVersion: v1
kind: Secret
metadata:
  name: auth-secrets
type: Opaque
stringData:
  jwt-secret: "your-super-secret-jwt-key-min-32-characters"
  smtp-password: "your-smtp-password"
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: auth-config
data:
  config.toml: |
    # Your production config.toml here
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: auth-data-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```

### Ingress
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: passwordless-auth
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - auth.yourapp.com
    secretName: auth-tls
  rules:
  - host: auth.yourapp.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: passwordless-auth
            port:
              number: 80
```

## 📊 **Monitoring Setup**

### Prometheus Configuration
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'passwordless-auth'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
```

### Grafana Dashboard JSON
Create a dashboard monitoring:
- Request rate
- Auth attempts (success/failure)
- Active sessions
- Email delivery rate
- Request latency (p50, p95, p99)
- Error rate

### Alert Rules
```yaml
# alerting_rules.yml
groups:
  - name: passwordless_auth
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(auth_attempts_total{status="failure"}[5m]) > 10
        for: 5m
        annotations:
          summary: "High authentication failure rate"

      - alert: ServiceDown
        expr: up{job="passwordless-auth"} == 0
        for: 1m
        annotations:
          summary: "Passwordless auth service is down"

      - alert: HighLatency
        expr: histogram_quantile(0.99, http_request_duration_seconds_bucket) > 1
        for: 5m
        annotations:
          summary: "99th percentile latency > 1s"
```

## 🔐 **Security Hardening**

### Reverse Proxy (Nginx)
```nginx
upstream auth_backend {
    server localhost:3000;
    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name auth.yourapp.com;

    # SSL configuration
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers (redundant but good practice)
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=auth_limit:10m rate=10r/s;
    limit_req zone=auth_limit burst=20 nodelay;

    location / {
        proxy_pass http://auth_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 10s;
    }

    # Metrics endpoint (restrict access)
    location /metrics {
        allow 10.0.0.0/8;  # Internal network
        deny all;
        proxy_pass http://auth_backend;
    }

    # Admin endpoints (restrict access)
    location /admin {
        allow 10.0.0.0/8;  # Internal network
        deny all;
        proxy_pass http://auth_backend;
    }
}
```

### Firewall Rules
```bash
# UFW example
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow from 10.0.0.0/8 to any port 3000  # Internal only
sudo ufw enable
```

## 💾 **Database Management**

### Backup Script
```bash
#!/bin/bash
# backup.sh

DB_PATH="/data/auth.db"
BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
sqlite3 $DB_PATH ".backup $BACKUP_DIR/auth_$DATE.db"

# Compress
gzip $BACKUP_DIR/auth_$DATE.db

# Keep only last 30 days
find $BACKUP_DIR -name "auth_*.db.gz" -mtime +30 -delete

echo "Backup completed: auth_$DATE.db.gz"
```

### Cron Job
```bash
# Run daily at 2 AM
0 2 * * * /path/to/backup.sh >> /var/log/auth-backup.log 2>&1
```

### Database Migrations
```bash
# Migrations run automatically on startup
# To run manually:
sqlite3 auth.db < migrations/init.sql
sqlite3 auth.db < migrations/002_email_queue.sql
sqlite3 auth.db < migrations/003_production_features.sql
```

## 📝 **Logging**

### Log Aggregation (Loki)
```yaml
# promtail-config.yml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: passwordless-auth
    static_configs:
      - targets:
          - localhost
        labels:
          job: auth
          __path__: /var/log/passwordless-auth/*.log
```

### Structured Logging
The application outputs JSON logs when LOG_FORMAT=json:
```json
{
  "timestamp": "2025-11-16T10:30:45Z",
  "level": "INFO",
  "target": "passwordless_auth",
  "fields": {
    "message": "User authenticated successfully",
    "user_id": "123e4567-e89b-12d3-a456-426614174000",
    "method": "magic_link",
    "ip": "192.168.1.1"
  }
}
```

## 🧪 **Health Checks**

### Endpoints
- `GET /health` - Application health + uptime
- `GET /readiness` - Ready to serve traffic
- `GET /liveness` - Application is alive

### Example Responses
```bash
# Health
curl http://localhost:3000/health
{
  "status": "healthy",
  "version": "0.2.0",
  "uptime_seconds": 3600,
  "timestamp": 1700140245
}

# Metrics
curl http://localhost:3000/metrics
# HELP auth_attempts_total Total authentication attempts
# TYPE auth_attempts_total counter
auth_attempts_total{method="magic_link",status="success"} 1234
...
```

## 🔄 **Rolling Updates**

### Zero-Downtime Deployment
```bash
# 1. Build new version
docker build -t passwordless-auth:0.2.1 .

# 2. Update deployment
kubectl set image deployment/passwordless-auth auth=passwordless-auth:0.2.1

# 3. Monitor rollout
kubectl rollout status deployment/passwordless-auth

# 4. Rollback if needed
kubectl rollout undo deployment/passwordless-auth
```

## 📞 **Support & Troubleshooting**

### Common Issues

**Issue**: High memory usage
**Solution**: Check for database connection leaks, adjust limits

**Issue**: Authentication failures
**Solution**: Check audit logs: `SELECT * FROM audit_logs WHERE success=0 ORDER BY created_at DESC LIMIT 100`

**Issue**: Email delivery failures
**Solution**: Check SMTP credentials, review email queue table

### Debug Mode
```bash
# Enable debug logging
export LOG_LEVEL=debug
export RUST_LOG=debug

# Or in config.toml
log_level = "debug"
```

### Useful Commands
```bash
# View active sessions
curl http://localhost:3000/admin/stats

# Revoke all sessions for a user
curl -X DELETE http://localhost:3000/admin/users/{user_id}/sessions

# Query audit logs
sqlite3 auth.db "SELECT * FROM audit_logs WHERE event_type='magic_link_verified' LIMIT 10"

# Check rate limit hits
curl http://localhost:3000/metrics | grep rate_limit
```

## 📈 **Scaling Considerations**

### Horizontal Scaling
- Application is stateless
- Multiple instances can run behind load balancer
- Sessions stored in database (shared state)

### Database Scaling
- Consider PostgreSQL for high load
- Implement read replicas
- Add caching layer (Redis)

### Performance Tuning
- Adjust rate limits based on traffic
- Monitor p99 latency
- Scale pods/containers based on CPU/memory
- Add CDN for static assets

---

**Last Updated**: 2025-11-16
**Version**: 0.2.0
**Status**: Production Ready (pending compilation fixes)
