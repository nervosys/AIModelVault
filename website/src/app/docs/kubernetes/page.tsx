import CodeBlock from "@/components/DocElements";
import { Callout } from "@/components/DocElements";

export default function KubernetesPage() {
  return (
    <>
      <h1 className="text-4xl font-bold mb-4">Kubernetes</h1>
      <p className="text-lg text-[var(--color-text-secondary)] mb-8">
        Deploy AI Model Vault on Kubernetes using the official Helm chart.
      </p>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="install">Helm Installation</h2>
      <CodeBlock language="bash">{`helm install ai-vault ./deploy/helm/ai-model-vault \\
  --namespace ai-vault \\
  --create-namespace \\
  --set vault.passphrase="your-vault-passphrase" \\
  --set vault.jwtSecret="your-jwt-secret"`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="values">Key Configuration Values</h2>
      <div className="overflow-x-auto">
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="border-b border-[var(--color-border)]">
              <th className="text-left p-3 font-semibold">Value</th>
              <th className="text-left p-3 font-semibold">Default</th>
              <th className="text-left p-3 font-semibold">Description</th>
            </tr>
          </thead>
          <tbody className="text-[var(--color-text-secondary)]">
            {[
              ["replicaCount", "1", "Number of pod replicas"],
              ["image.repository", "ghcr.io/nervosys/ai-model-vault", "Container image repo"],
              ["image.tag", "latest", "Image tag"],
              ["service.type", "ClusterIP", "Service type"],
              ["service.port", "8080", "Service port"],
              ["ingress.enabled", "false", "Enable ingress"],
              ["persistence.enabled", "true", "Enable persistent volume"],
              ["persistence.size", "10Gi", "PVC size"],
              ["autoscaling.enabled", "false", "Enable HPA"],
              ["autoscaling.minReplicas", "1", "Minimum replicas"],
              ["autoscaling.maxReplicas", "10", "Maximum replicas"],
              ["vault.passphrase", '""', "Vault passphrase (stored in Secret)"],
              ["vault.jwtSecret", '""', "JWT signing secret"],
            ].map(([name, def, desc]) => (
              <tr key={name} className="border-b border-[var(--color-border)]">
                <td className="p-3"><code className="text-xs">{name}</code></td>
                <td className="p-3"><code className="text-xs">{def}</code></td>
                <td className="p-3">{desc}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <Callout type="warning" title="Secrets management">
        Never commit passphrases or JWT secrets in values.yaml. Use <code className="text-xs">--set</code>,
        environment variables, or external secrets management (Vault, SOPS, Sealed Secrets).
      </Callout>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="ingress">Ingress with TLS</h2>
      <CodeBlock language="yaml" title="values-production.yaml">{`ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: vault.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: vault-tls
      hosts:
        - vault.example.com`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="hpa">Autoscaling</h2>
      <CodeBlock language="yaml" title="values-production.yaml">{`autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80`}</CodeBlock>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="security">Security Context</h2>
      <p className="text-[var(--color-text-secondary)] mb-4">
        Pods run with a hardened security context by default:
      </p>
      <ul className="space-y-1 text-[var(--color-text-secondary)]">
        <li>• Non-root user (UID 1000)</li>
        <li>• Read-only root filesystem</li>
        <li>• All Linux capabilities dropped</li>
        <li>• Privilege escalation disabled</li>
      </ul>

      <h2 className="text-2xl font-bold mt-10 mb-4" id="upgrade">Upgrading</h2>
      <CodeBlock language="bash">{`helm upgrade ai-vault ./deploy/helm/ai-model-vault \\
  --namespace ai-vault \\
  --reuse-values`}</CodeBlock>
    </>
  );
}
