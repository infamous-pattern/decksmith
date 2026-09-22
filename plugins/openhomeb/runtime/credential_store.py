"""Secret Service adapter. No plaintext fallback; never pass secrets in argv."""
import asyncio
import re

class CredentialStoreError(RuntimeError): pass

class SecretToolStore:
    def __init__(self, runner=None): self.runner=runner or self._run
    def attributes(self, profile):
        if not re.fullmatch(r'[a-zA-Z0-9_-]{1,64}',profile):raise ValueError('invalid profile ID')
        return ['application','decksmith-homebridge','profile',profile]
    async def _run(self, args, secret=None):
        try:
            child=await asyncio.create_subprocess_exec('/usr/bin/secret-tool',*args,
                stdin=asyncio.subprocess.PIPE,stdout=asyncio.subprocess.PIPE,stderr=asyncio.subprocess.DEVNULL)
        except OSError:raise CredentialStoreError('Desktop secret store unavailable') from None
        try:
            output,_=await asyncio.wait_for(child.communicate(secret.encode() if secret is not None else None),10)
        except BaseException:
            try: child.kill()
            except ProcessLookupError: pass
            await child.wait();raise
        return child.returncode,output
    async def lookup(self, profile):
        code,out=await self.runner(['lookup',*self.attributes(profile)])
        if code==1:return None  # Missing or locked: never fall back to a file.
        if code!=0:raise CredentialStoreError('Could not read desktop secret store')
        return out.decode().removesuffix('\n')
    async def store(self, profile, password):
        if not isinstance(password,str) or not 0<len(password)<=4096 or any(c in password for c in ('\n','\r','\x00')):raise ValueError('invalid password')
        code,_=await self.runner(['store','--label=Decksmith Homebridge',*self.attributes(profile)],password)
        if code:raise CredentialStoreError('Could not save to desktop secret store')
    async def delete(self, profile):
        code,_=await self.runner(['clear',*self.attributes(profile)])
        if code:raise CredentialStoreError('Could not remove desktop secret-store entry')
