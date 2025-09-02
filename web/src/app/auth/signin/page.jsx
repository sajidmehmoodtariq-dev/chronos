'use client';

import { signIn, getProviders, useSession } from 'next-auth/react';
import { useState, useEffect } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import Link from 'next/link';

export default function SignIn() {
  const [providers, setProviders] = useState(null);
  const { data: session, status } = useSession();
  const router = useRouter();
  const searchParams = useSearchParams();
  
  const isFromDesktop = searchParams.get('source') === 'desktop';

  useEffect(() => {
    const setAuthProviders = async () => {
      const res = await getProviders();
      setProviders(res);
    };
    setAuthProviders();
  }, []);

  useEffect(() => {
    if (status === 'authenticated') {
      // Redirect to token page if coming from desktop
      if (isFromDesktop) {
        router.push('/token?source=desktop');
      } else {
        router.push('/dashboard');
      }
    }
  }, [status, router, isFromDesktop]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
      <div className="bg-white p-8 rounded-lg shadow-md w-full max-w-md">
        <div className="text-center mb-8">
          {isFromDesktop ? (
            <>
              <div className="text-6xl mb-4">🚀</div>
              <h1 className="text-3xl font-bold text-gray-900 mb-2">Welcome to Chronos!</h1>
              <p className="text-gray-600">Complete your desktop setup by signing in below</p>
            </>
          ) : (
            <>
              <h1 className="text-3xl font-bold text-gray-900 mb-2">Welcome to Chronos</h1>
              <p className="text-gray-600">Sign in to access your activity dashboard</p>
            </>
          )}
        </div>

        <div className="space-y-4">
          {providers &&
            Object.values(providers).map((provider) => (
              <button
                key={provider.name}
                onClick={() => signIn(provider.id, { callbackUrl: '/dashboard' })}
                className="w-full flex items-center justify-center px-4 py-3 border border-gray-300 rounded-md shadow-sm bg-white text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors"
              >
                {provider.name === 'Google' && '🌐'}
                {provider.name === 'GitHub' && '🐙'}
                <span className="ml-2">Continue with {provider.name}</span>
              </button>
            ))}
        </div>

        {isFromDesktop && (
          <div className="mt-6 p-4 bg-blue-50 rounded-lg">
            <p className="text-sm text-blue-800">
              <strong>Next:</strong> After signing in, you'll automatically get your sync token to complete the desktop setup.
            </p>
          </div>
        )}

        <div className="mt-8 text-center">
          <Link href="/" className="text-blue-600 hover:underline text-sm">
            ← Back to home
          </Link>
        </div>
      </div>
    </div>
  );
}
