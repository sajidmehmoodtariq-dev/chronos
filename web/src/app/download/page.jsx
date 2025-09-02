export default function DownloadPage() {
  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white shadow">
        <div className="container mx-auto px-4 py-4 flex justify-between items-center">
          <h1 className="text-2xl font-bold text-gray-900">Download Chronos</h1>
          <div className="flex items-center space-x-4">
            <a 
              href="/"
              className="text-blue-600 hover:text-blue-800"
            >
              ← Back to Home
            </a>
            <a
              href="/auth/signin"
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded text-sm transition-colors"
            >
              Sign In
            </a>
          </div>
        </div>
      </div>

      {/* Download Section */}
      <div className="container mx-auto px-4 py-16">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-4xl font-bold text-gray-900 mb-6">
            Download Chronos Activity Tracker
          </h2>
          <p className="text-xl text-gray-600 mb-12">
            Track your digital activity automatically. Available for Windows.
          </p>

          <div className="grid md:grid-cols-2 gap-8 mb-12">
            {/* Windows Download */}
            <div className="bg-white rounded-lg shadow-lg p-8">
              <div className="text-6xl mb-4">🪟</div>
              <h3 className="text-2xl font-semibold text-gray-900 mb-4">Windows</h3>
              <p className="text-gray-600 mb-6">
                Full installer with automatic startup and system integration.
              </p>
              <button className="bg-blue-600 hover:bg-blue-700 text-white px-8 py-3 rounded-lg text-lg font-semibold transition-colors mb-4">
                Download Installer (v1.0.0)
              </button>
              <div className="text-sm text-gray-500">
                <p>Windows 10/11 • 64-bit • 2.5 MB</p>
              </div>
            </div>

            {/* Manual Download */}
            <div className="bg-white rounded-lg shadow-lg p-8">
              <div className="text-6xl mb-4">📦</div>
              <h3 className="text-2xl font-semibold text-gray-900 mb-4">Portable</h3>
              <p className="text-gray-600 mb-6">
                Single executable file - no installation required.
              </p>
              <button className="bg-gray-600 hover:bg-gray-700 text-white px-8 py-3 rounded-lg text-lg font-semibold transition-colors mb-4">
                Download EXE Only
              </button>
              <div className="text-sm text-gray-500">
                <p>Windows 10/11 • 64-bit • 1.8 MB</p>
              </div>
            </div>
          </div>

          {/* Installation Instructions */}
          <div className="bg-white rounded-lg shadow p-8 text-left">
            <h3 className="text-xl font-semibold text-gray-900 mb-4">📋 Installation Instructions</h3>
            <div className="space-y-4">
              <div className="flex items-start space-x-3">
                <span className="bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center text-sm font-bold">1</span>
                <div>
                  <p className="font-medium">Download and run the installer</p>
                  <p className="text-gray-600 text-sm">The installer will guide you through the setup process.</p>
                </div>
              </div>
              <div className="flex items-start space-x-3">
                <span className="bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center text-sm font-bold">2</span>
                <div>
                  <p className="font-medium">Sign in to your account</p>
                  <p className="text-gray-600 text-sm">Create an account or sign in with Google/GitHub.</p>
                </div>
              </div>
              <div className="flex items-start space-x-3">
                <span className="bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center text-sm font-bold">3</span>
                <div>
                  <p className="font-medium">Get your sync token</p>
                  <p className="text-gray-600 text-sm">Go to your dashboard and generate a sync token for the app.</p>
                </div>
              </div>
              <div className="flex items-start space-x-3">
                <span className="bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center text-sm font-bold">4</span>
                <div>
                  <p className="font-medium">Start tracking</p>
                  <p className="text-gray-600 text-sm">Chronos will start automatically and sync your activity to the dashboard.</p>
                </div>
              </div>
            </div>
          </div>

          {/* System Requirements */}
          <div className="mt-8 bg-gray-100 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-3">💻 System Requirements</h3>
            <div className="text-sm text-gray-600 space-y-1">
              <p>• Windows 10 or Windows 11</p>
              <p>• 64-bit processor</p>
              <p>• 50 MB free disk space</p>
              <p>• Internet connection for sync</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
