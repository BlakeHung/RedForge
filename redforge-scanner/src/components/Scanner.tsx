import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Play, AlertTriangle, CheckCircle, XCircle, Loader2 } from 'lucide-react';

interface ScanTask {
  id: string;
  target_url: string;
  scan_type: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  started_at?: string;
  completed_at?: string;
  created_at: string;
}

function Scanner() {
  const [url, setUrl] = useState('https://wchung.tw');
  const [scanType, setScanType] = useState('full');
  const [isScanning, setIsScanning] = useState(false);
  const [currentTask, setCurrentTask] = useState<ScanTask | null>(null);

  const startScan = async () => {
    if (!url) {
      alert('請輸入目標 URL');
      return;
    }

    setIsScanning(true);

    try {
      const taskId = await invoke<string>('start_scan', {
        url,
        scanType,
      });

      console.log('Scan started:', taskId);

      // 輪詢掃描狀態
      const pollInterval = setInterval(async () => {
        try {
          const task = await invoke<ScanTask>('get_scan_status', {
            taskId,
          });

          setCurrentTask(task);

          if (task.status === 'completed' || task.status === 'failed') {
            clearInterval(pollInterval);
            setIsScanning(false);

            if (task.status === 'completed') {
              // 獲取掃描結果
              console.log('Scan completed');
            }
          }
        } catch (err) {
          console.error('Failed to poll status:', err);
        }
      }, 1000);
    } catch (error) {
      console.error('Failed to start scan:', error);
      setIsScanning(false);
      alert('掃描啟動失敗: ' + error);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running':
        return <Loader2 className="w-5 h-5 text-info-500 animate-spin" />;
      case 'completed':
        return <CheckCircle className="w-5 h-5 text-success-500" />;
      case 'failed':
        return <XCircle className="w-5 h-5 text-danger-500" />;
      default:
        return <AlertTriangle className="w-5 h-5 text-warning-500" />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Scan Configuration Card */}
      <div className="bg-dark-800 rounded-lg border border-dark-700 p-6">
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center">
          <Play className="w-6 h-6 mr-2 text-danger-500" />
          啟動掃描
        </h2>

        <div className="space-y-4">
          {/* URL Input */}
          <div>
            <label className="block text-sm font-medium text-dark-300 mb-2">
              目標 URL
            </label>
            <input
              type="url"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://example.com"
              className="w-full px-4 py-3 bg-dark-700 border border-dark-600 rounded-lg text-white placeholder-dark-400 focus:outline-none focus:ring-2 focus:ring-danger-500"
            />
          </div>

          {/* Scan Type Selection */}
          <div>
            <label className="block text-sm font-medium text-dark-300 mb-2">
              掃描類型
            </label>
            <div className="grid grid-cols-3 gap-3">
              {[
                { id: 'quick', label: '快速掃描', desc: '基本安全檢查' },
                { id: 'full', label: '完整掃描', desc: 'Headers + SSL + 漏洞' },
                { id: 'vulnerability', label: '漏洞掃描', desc: 'OWASP Top 10' },
              ].map((type) => (
                <button
                  key={type.id}
                  onClick={() => setScanType(type.id)}
                  className={`p-4 rounded-lg border-2 text-left transition-all ${
                    scanType === type.id
                      ? 'border-danger-500 bg-danger-900/20'
                      : 'border-dark-600 bg-dark-700 hover:border-dark-500'
                  }`}
                >
                  <div className="font-semibold text-white">{type.label}</div>
                  <div className="text-xs text-dark-400 mt-1">{type.desc}</div>
                </button>
              ))}
            </div>
          </div>

          {/* Start Button */}
          <button
            onClick={startScan}
            disabled={isScanning}
            className={`w-full py-3 rounded-lg font-semibold transition-all ${
              isScanning
                ? 'bg-dark-600 text-dark-400 cursor-not-allowed'
                : 'bg-danger-600 hover:bg-danger-700 text-white'
            }`}
          >
            {isScanning ? (
              <span className="flex items-center justify-center">
                <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                掃描中...
              </span>
            ) : (
              <span className="flex items-center justify-center">
                <Play className="w-5 h-5 mr-2" />
                開始掃描
              </span>
            )}
          </button>
        </div>
      </div>

      {/* Current Task Status */}
      {currentTask && (
        <div className="bg-dark-800 rounded-lg border border-dark-700 p-6">
          <h3 className="text-lg font-semibold text-white mb-4 flex items-center">
            {getStatusIcon(currentTask.status)}
            <span className="ml-2">掃描狀態</span>
          </h3>

          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-dark-400">任務 ID:</span>
              <span className="text-white font-mono">{currentTask.id.slice(0, 8)}...</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-dark-400">目標:</span>
              <span className="text-white">{currentTask.target_url}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-dark-400">掃描類型:</span>
              <span className="text-white uppercase">{currentTask.scan_type}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-dark-400">狀態:</span>
              <span className={`font-semibold uppercase ${
                currentTask.status === 'completed' ? 'text-success-500' :
                currentTask.status === 'failed' ? 'text-danger-500' :
                currentTask.status === 'running' ? 'text-info-500' :
                'text-warning-500'
              }`}>
                {currentTask.status}
              </span>
            </div>
            {currentTask.started_at && (
              <div className="flex justify-between text-sm">
                <span className="text-dark-400">開始時間:</span>
                <span className="text-white">{new Date(currentTask.started_at).toLocaleString('zh-TW')}</span>
              </div>
            )}
          </div>

          {/* Progress Animation */}
          {currentTask.status === 'running' && (
            <div className="mt-4">
              <div className="h-2 bg-dark-700 rounded-full overflow-hidden">
                <div className="h-full bg-gradient-to-r from-danger-600 to-danger-400 animate-pulse" style={{ width: '60%' }}></div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Warning Card */}
      <div className="bg-warning-900/20 border border-warning-700 rounded-lg p-4">
        <div className="flex items-start">
          <AlertTriangle className="w-5 h-5 text-warning-500 mr-3 mt-0.5 flex-shrink-0" />
          <div className="text-sm text-warning-200">
            <p className="font-semibold mb-1">重要提醒</p>
            <p>請確保您有權限掃描目標網站。未經授權的安全測試可能違反法律。</p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Scanner;
