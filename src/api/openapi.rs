//! OpenAPI specification generator.

/// Minimal OpenAPI 3.0 spec for the vault API.
///
/// Returns a JSON-serialisable structure.  We generate the spec procedurally
/// rather than depending on a derive macro so the `api` feature stays light.
pub fn openapi_spec() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "AI Model Vault API",
            "description": "RESTful API for secure AI model storage, versioning, and format conversion.",
            "version": "0.5.0",
            "license": {
                "name": "AGPL-3.0-or-later",
                "url": "https://www.gnu.org/licenses/agpl-3.0.html"
            }
        },
        "servers": [
            { "url": "/api/v1", "description": "Local server" }
        ],
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "tags": ["system"],
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/auth/token": {
                "post": {
                    "summary": "Obtain JWT token",
                    "tags": ["auth"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "passphrase": { "type": "string" }
                                    },
                                    "required": ["passphrase"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": { "description": "JWT token" },
                        "401": { "description": "Invalid passphrase" }
                    }
                }
            },
            "/models": {
                "get": {
                    "summary": "List models in the vault",
                    "tags": ["models"],
                    "security": [{ "bearerAuth": [] }],
                    "responses": { "200": { "description": "Array of model names" } }
                }
            },
            "/models/{name}": {
                "get": {
                    "summary": "Retrieve latest model data",
                    "tags": ["models"],
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "name": "name", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": {
                        "200": { "description": "Model binary data" },
                        "404": { "description": "Model not found" }
                    }
                },
                "post": {
                    "summary": "Store a model (multipart upload)",
                    "tags": ["models"],
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "name": "name", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "multipart/form-data": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "file": { "type": "string", "format": "binary" },
                                        "format": { "type": "string" },
                                        "description": { "type": "string" }
                                    },
                                    "required": ["file", "format"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": { "description": "Version created" }
                    }
                }
            },
            "/models/{name}/versions": {
                "get": {
                    "summary": "List versions of a model",
                    "tags": ["versions"],
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "name": "name", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": { "200": { "description": "Array of versions" } }
                }
            },
            "/models/{name}/versions/{version}": {
                "get": {
                    "summary": "Retrieve specific model version data",
                    "tags": ["versions"],
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "name": "name", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "version", "in": "path", "required": true, "schema": { "type": "integer" } }
                    ],
                    "responses": {
                        "200": { "description": "Model binary data" },
                        "404": { "description": "Version not found" }
                    }
                },
                "delete": {
                    "summary": "Delete a model version",
                    "tags": ["versions"],
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "name": "name", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "version", "in": "path", "required": true, "schema": { "type": "integer" } }
                    ],
                    "responses": {
                        "200": { "description": "Deleted" },
                        "404": { "description": "Version not found" }
                    }
                }
            },
            "/models/{name}/lineage/{version}": {
                "get": {
                    "summary": "Get version lineage/history",
                    "tags": ["versions"],
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "name": "name", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "version", "in": "path", "required": true, "schema": { "type": "integer" } }
                    ],
                    "responses": { "200": { "description": "Lineage array" } }
                }
            },
            "/conversions": {
                "get": {
                    "summary": "List supported format conversions",
                    "tags": ["conversions"],
                    "responses": { "200": { "description": "Conversion list" } }
                }
            },
            "/convert": {
                "post": {
                    "summary": "Convert model data between formats",
                    "tags": ["conversions"],
                    "security": [{ "bearerAuth": [] }],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "data_base64": { "type": "string" },
                                        "source_format": { "type": "string" },
                                        "target_format": { "type": "string" },
                                        "quantization": { "type": "string" },
                                        "opset_version": { "type": "integer" },
                                        "validate": { "type": "boolean" }
                                    },
                                    "required": ["data_base64", "source_format", "target_format"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": { "description": "Conversion result" }
                    }
                }
            },
            "/stats": {
                "get": {
                    "summary": "Vault statistics",
                    "tags": ["system"],
                    "security": [{ "bearerAuth": [] }],
                    "responses": { "200": { "description": "Vault stats" } }
                }
            },
            "/audit": {
                "get": {
                    "summary": "Audit log entries",
                    "tags": ["system"],
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "name": "limit", "in": "query", "schema": { "type": "integer" } }
                    ],
                    "responses": { "200": { "description": "Audit entries" } }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        }
    })
}
