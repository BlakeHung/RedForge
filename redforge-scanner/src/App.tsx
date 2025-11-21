import { useState } from 'react';
import './styles/globals.css';
import Scanner from './components/Scanner';
import ScanHistory from './components/ScanHistory';
import Dashboard from './components/Dashboard';
import { Shield } from 'lucide-react';

type TabType = 'scanner' | 'history' | 'dashboard';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('scanner');

  return (
    <div className="min-h-screen bg-dark-900">
      {/* Header */}
      <header className="bg-dark-800 border-b border-dark-700 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-3">
              <Shield className="w-8 h-8 text-danger-500" />
              <div>
                <h1 className="text-xl font-bold text-white">RedForge Scanner</h1>
                <p className="text-xs text-dark-400">紅隊安全掃描系統</p>
              </div>
            </div>

            <nav className="flex space-x-1">
              {[
                { id: 'scanner', label: '掃描器', icon: '🔍' },
                { id: 'history', label: '歷史記錄', icon: '📋' },
                { id: 'dashboard', label: '儀表板', icon: '📊' },
              ].map(tab => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as TabType)}
                  className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                    activeTab === tab.id
                      ? 'bg-danger-600 text-white'
                      : 'text-dark-300 hover:bg-dark-700 hover:text-white'
                  }`}
                >
                  <span className="mr-2">{tab.icon}</span>
                  {tab.label}
                </button>
              ))}
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {activeTab === 'scanner' && <Scanner />}
        {activeTab === 'history' && <ScanHistory />}
        {activeTab === 'dashboard' && <Dashboard />}
      </main>

      {/* Footer */}
      <footer className="bg-dark-800 border-t border-dark-700 mt-auto py-4">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-dark-400 text-sm">
          <p>⚠️ 僅用於授權測試 | Target: wchung.tw</p>
        </div>
      </footer>
    </div>
  );
}

export default App;
