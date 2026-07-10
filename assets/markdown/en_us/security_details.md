# Security Details

Safe Mode is designed to improve the security of third-party scripts during execution.

When Safe Mode is enabled, access to certain high-risk APIs is restricted to reduce the possibility of scripts affecting local data or the system environment.

Disabling Safe Mode does not grant scripts full system access. All scripts continue to run inside the runtime sandbox and remain subject to the restrictions imposed by the application environment. Safe Mode serves as an additional security layer that further limits access to sensitive capabilities.

If you trust the current script or mod, you may disable Safe Mode to unlock its full functionality.

# Capabilities Restricted by Safe Mode

## File Write APIs

Restricts scripts from creating, modifying, or deleting local files.

[API list to be added]

## System APIs

Restricts access to certain system functions and interactions with the host environment.

[API list to be added]

## Network APIs

Restricts scripts from initiating network requests or accessing external resources.

[API list to be added]
