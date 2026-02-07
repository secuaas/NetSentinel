"""
SecretLoader for Python applications
Loads secrets from OVH Secret Manager with fallback to environment variables
"""
import os
from typing import Dict, Optional


class SecretLoader:
    """Manages loading secrets from OVH Secret Manager with env var fallback"""
    
    def __init__(self, app_name: str = "netsentinel"):
        self.app_name = app_name
        self.cache: Dict[str, str] = {}
        self.client = None
        
        # Try to load OVH SM client
        # TODO: Implement OVH SM client for Python when available
        # For now, using environment variables only
        
    def load_secrets(self, environment: str = "dev") -> Dict[str, str]:
        """Load all secrets for the environment"""
        if self.client:
            # Try OVH SM first (when implemented)
            try:
                path = f"apps/{self.app_name}/{environment}"
                secrets = self.client.get_secrets(path)
                print(f"[SecretLoader] Loaded {len(secrets)} secrets from OVH SM")
                self.cache = secrets
                return secrets
            except Exception as e:
                print(f"[SecretLoader] Warning: OVH SM failed: {e}")
        
        # Fallback to environment variables
        secrets = {k: v for k, v in os.environ.items() if v}
        if secrets:
            print(f"[SecretLoader] Loaded {len(secrets)} secrets from environment")
        self.cache = secrets
        return secrets
    
    def get_secret(self, name: str, environment: str = "dev", fallback: str = "") -> str:
        """Get a single secret with fallback"""
        if not self.cache:
            self.load_secrets(environment)
        
        return self.cache.get(name) or os.getenv(name) or fallback
    
    def get_secret_required(self, name: str, environment: str = "dev") -> str:
        """Get a required secret, raise error if not found"""
        value = self.get_secret(name, environment)
        if not value:
            raise ValueError(f"Required secret {name} not found")
        return value


# Global instance (singleton pattern)
_loader: Optional[SecretLoader] = None


def get_loader(app_name: str = "netsentinel") -> SecretLoader:
    """Get or create the global SecretLoader instance"""
    global _loader
    if _loader is None:
        _loader = SecretLoader(app_name)
    return _loader
