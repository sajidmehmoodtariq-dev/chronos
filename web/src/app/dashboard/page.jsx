'use client';

import { useSession, signOut } from 'next-auth/react';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

export default function Dashboard() {
  const { data: session, status } = useSession();
  const router = useRouter();
  const [logs, setLogs] = useState([]);
  const [loading, setLoading] = useState(true);
  const [metrics, setMetrics] = useState({
    totalEvents: 0,
    activeHours: 0,
    appsUsed: 0
  });

  useEffect(() => {
    if (status === 'unauthenticated') {
      router.push('/auth/signin');
    }
    if (status === 'authenticated') {
      fetchLogs();
    }
  }, [status, router]);

  const calculateMetrics = (logsData) => {
    if (!logsData || logsData.length === 0) {
      return { totalEvents: 0, activeHours: 0, appsUsed: 0 };
    }

    // Calculate unique apps used
    const uniqueApps = new Set();
    logsData.forEach(log => {
      if (log.type === 'window' && log.data?.processName) {
        uniqueApps.add(log.data.processName);
      }
    });

    // Calculate active hours (simplified - count unique hours with activity)
    const today = new Date().toDateString();
    const todayLogs = logsData.filter(log => 
      new Date(log.timestamp).toDateString() === today
    );
    
    const activeHours = new Set();
    todayLogs.forEach(log => {
      const hour = new Date(log.timestamp).getHours();
      activeHours.add(hour);
    });

    return {
      totalEvents: logsData.length,
      activeHours: activeHours.size,
      appsUsed: uniqueApps.size
    };
  };

  const fetchLogs = async () => {
    try {
      const res = await fetch('/api/logs');
      if (res.ok) {
        const data = await res.json();
        setLogs(data);
        setMetrics(calculateMetrics(data));
      }
    } catch (error) {
      console.error('Error fetching logs:', error);
    } finally {
      setLoading(false);
    }
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
          <h1 className="text-2xl font-bold text-gray-900">Chronos Dashboard</h1>
          <div className="flex items-center space-x-4">
            <span className="text-gray-600">Welcome, {session.user.name}</span>
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
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          <div className="bg-white p-6 rounded-lg shadow">
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Total Activity</h3>
            <p className="text-3xl font-bold text-blue-600">{metrics.totalEvents}</p>
            <p className="text-gray-600 text-sm">events logged</p>
          </div>
          
          <div className="bg-white p-6 rounded-lg shadow">
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Active Hours</h3>
            <p className="text-3xl font-bold text-green-600">{metrics.activeHours}h</p>
            <p className="text-gray-600 text-sm">today</p>
          </div>
          
          <div className="bg-white p-6 rounded-lg shadow">
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Apps Used</h3>
            <p className="text-3xl font-bold text-purple-600">{metrics.appsUsed}</p>
            <p className="text-gray-600 text-sm">different applications</p>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">Recent Activity</h3>
          </div>
          <div className="p-6">
            {loading ? (
              <div className="text-center py-8">
                <div className="text-gray-600">Loading activity logs...</div>
              </div>
            ) : logs.length === 0 ? (
              <div className="text-center py-8">
                <div className="text-gray-600">No activity logs found.</div>
                <p className="text-sm text-gray-500 mt-2">
                  Make sure the Chronos tracker is running on your computer.
                </p>
              </div>
            ) : (
              <div className="space-y-4">
                {logs.slice(0, 10).map((log, index) => (
                  <div key={index} className="flex items-start space-x-4 p-4 border border-gray-200 rounded-lg">
                    <div className="text-2xl">
                      {log.type === 'window' && '🖥️'}
                      {log.type === 'browser' && '🌐'}
                      {log.type === 'keyboard' && '⌨️'}
                      {log.type === 'mouse' && '🖱️'}
                    </div>
                    <div className="flex-1">
                      <div className="flex justify-between items-start">
                        <div>
                          <h4 className="font-medium text-gray-900">
                            {log.type === 'window' && `${log.data.processName}: ${log.data.windowTitle}`}
                            {log.type === 'browser' && `${log.data.browserTitle}`}
                            {log.type === 'keyboard' && 'Keyboard Activity'}
                            {log.type === 'mouse' && 'Mouse Activity'}
                          </h4>
                          {log.data.url && (
                            <p className="text-sm text-blue-600 truncate">{log.data.url}</p>
                          )}
                        </div>
                        <span className="text-sm text-gray-500">
                          {new Date(log.timestamp).toLocaleTimeString()}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
