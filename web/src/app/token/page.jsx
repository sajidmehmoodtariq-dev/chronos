'use client';

import { useSession, signOut } from 'next-auth/react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useEffect, useState } from 'react';

export default function TokenPage() {
  const { data: session, status } = useSession();
  const router = useRouter();
  const searchParams = useSearchParams();
  const [token, setToken] = useState('');
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  
  const isFromDesktop = searchParams.get('source') === 'desktop';

  useEffect(() => {
    if (status === 'unauthenticated') {
      router.push('/auth/signin');
    }
    
    // Auto-generate token for desktop users
    if (status === 'authenticated' && isFromDesktop && !token) {
      generateToken();
    }
  }, [status, router, isFromDesktop, token]);

  const generateToken = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/auth/token', {
        method: 'POST',
      });
      
      if (res.ok) {
        const data = await res.json();
        setToken(data.token);
      } else {
        alert('Failed to generate token');
      }
    } catch (error) {
      console.error('Error generating token:', error);
      alert('Error generating token');
    } finally {
      setLoading(false);
    }
  };

  const copyToken = () => {
    navigator.clipboard.writeText(token);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (status === 'loading') {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-lg">Loading...</div>
      </div>
    );
  }

  if (!session) {
    return null;
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="bg-white shadow">
        <div className="container mx-auto px-4 py-4 flex justify-between items-center">
          <h1 className="text-2xl font-bold text-gray-900">Sync Token</h1>
          <div className="flex items-center space-x-4">
            <a 
              href="/dashboard"
              className="text-blue-600 hover:text-blue-800"
            >
              ← Back to Dashboard
            </a>
            <button
              onClick={() => signOut()}
              className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded text-sm transition-colors"
            >
              Sign Out
            </button>
          </div>
        </div>
      </div>

      <div className="container mx-auto px-4 py-8">
        <div className="max-w-2xl mx-auto">
          <div className="bg-white rounded-lg shadow p-6">
            {isFromDesktop ? (
              <>
                <div className="text-center mb-6">
                  <div className="text-6xl mb-4">🔑</div>
                  <h2 className="text-2xl font-semibold text-gray-900 mb-2">
                    Desktop Setup Complete!
                  </h2>
                  <p className="text-gray-600">
                    Copy this token and paste it in your desktop app to finish setup.
                  </p>
                </div>
              </>
            ) : (
              <>
                <h2 className="text-xl font-semibold text-gray-900 mb-4">
                  Generate Sync Token for Rust Client
                </h2>
                <p className="text-gray-600 mb-6">
                  This token will allow your local Chronos tracker to sync data to your account. 
                  Keep it secure and don't share it with others.
                </p>
              </>
            )}

            {!token ? (
              <button
                onClick={generateToken}
                disabled={loading}
                className="bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white px-6 py-3 rounded-lg transition-colors"
              >
                {loading ? 'Generating...' : 'Generate Token'}
              </button>
            ) : (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Your Sync Token:
                </label>
                <div className="relative">
                  <textarea
                    value={token}
                    readOnly
                    className="w-full p-3 border border-gray-300 rounded-lg bg-gray-50 text-sm font-mono"
                    rows={4}
                  />
                  <button
                    onClick={copyToken}
                    className="absolute top-2 right-2 bg-gray-600 hover:bg-gray-700 text-white px-3 py-1 rounded text-xs transition-colors"
                  >
                    {copied ? 'Copied!' : 'Copy'}
                  </button>
                </div>
                <p className="text-sm text-gray-500 mt-2">
                  {isFromDesktop ? (
                    "Paste this token in your desktop app to complete setup."
                  ) : (
                    "Save this token and add it to your Rust client configuration."
                  )}
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
