import Link from "next/link";

export default function Home() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="container mx-auto px-4 py-16">
        <div className="text-center mb-16">
          <h1 className="text-5xl font-bold text-gray-900 mb-6">
            Chronos
          </h1>
          <p className="text-xl text-gray-600 mb-8 max-w-2xl mx-auto">
            Track your digital activity, monitor productivity, and gain insights into your computer usage patterns with our lightweight background tracker.
          </p>
        </div>

        <div className="grid md:grid-cols-3 gap-8 mb-16">
          <div className="bg-white p-6 rounded-lg shadow-md">
            <div className="text-blue-600 text-2xl mb-4">⌨️</div>
            <h3 className="text-xl font-semibold mb-2">Activity Tracking</h3>
            <p className="text-gray-600">Monitor keyboard and mouse activity to understand your active hours.</p>
          </div>
          
          <div className="bg-white p-6 rounded-lg shadow-md">
            <div className="text-blue-600 text-2xl mb-4">🖥️</div>
            <h3 className="text-xl font-semibold mb-2">App Monitoring</h3>
            <p className="text-gray-600">Track which applications you use and for how long.</p>
          </div>
          
          <div className="bg-white p-6 rounded-lg shadow-md">
            <div className="text-blue-600 text-2xl mb-4">🌐</div>
            <h3 className="text-xl font-semibold mb-2">Browser History</h3>
            <p className="text-gray-600">Log your browsing activity across Chrome, Firefox, Edge, and Brave.</p>
          </div>
        </div>

        <div className="text-center space-y-4">
          <div className="space-x-4">
            <Link 
              href="/auth/signin"
              className="bg-blue-600 hover:bg-blue-700 text-white px-8 py-3 rounded-lg font-medium inline-block transition-colors"
            >
              Get Started
            </Link>
            <a 
              href="#"
              className="bg-gray-200 hover:bg-gray-300 text-gray-800 px-8 py-3 rounded-lg font-medium inline-block transition-colors"
            >
              Download Installer
            </a>
          </div>
          <p className="text-sm text-gray-500">
            Already have an account? <Link href="/auth/signin" className="text-blue-600 hover:underline">Sign in</Link>
          </p>
        </div>
      </div>
    </div>
  );
}
