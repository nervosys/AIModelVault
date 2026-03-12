import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // MkDocs builds to public/mkdocs/ — Next.js serves it as static files at /mkdocs/
  // No rewrites needed: files in public/ are served as-is.
  //
  // Build flow:
  //   1. mkdocs build              (outputs to website/public/mkdocs/)
  //   2. cd website && next build  (bundles everything including /mkdocs/)
};

export default nextConfig;
