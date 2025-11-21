import { useState } from 'react';
import { BarChart3, Shield, AlertTriangle, CheckCircle, Info } from 'lucide-react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';

interface VulnerabilityStats {
  critical: number;
  high: number;
  medium: number;
  low: number;
  info: number;
}

function Dashboard() {
  const [stats] = useState<VulnerabilityStats>({
    critical: 2,
    high: 5,
    medium: 8,
    low: 12,
    info: 15,
  });

  const severityData = [
    { name: 'Critical', value: stats.critical, color: '#dc2626' },
    { name: 'High', value: stats.high, color: '#ea580c' },
    { name: 'Medium', value: stats.medium, color: '#ca8a04' },
    { name: 'Low', value: stats.low, color: '#16a34a' },
    { name: 'Info', value: stats.info, color: '#2563eb' },
  ];

  const barData = severityData.map(item => ({
    name: item.name,
    count: item.value,
  }));

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-white flex items-center">
        <BarChart3 className="w-6 h-6 mr-2 text-danger-500" />
        安全儀表板
      </h2>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        <StatCard
          title="Critical"
          value={stats.critical}
          icon={<Shield className="w-6 h-6" />}
          color="danger"
        />
        <StatCard
          title="High"
          value={stats.high}
          icon={<AlertTriangle className="w-6 h-6" />}
          color="warning"
        />
        <StatCard
          title="Medium"
          value={stats.medium}
          icon={<AlertTriangle className="w-6 h-6" />}
          color="warning"
          lighter
        />
        <StatCard
          title="Low"
          value={stats.low}
          icon={<CheckCircle className="w-6 h-6" />}
          color="success"
        />
        <StatCard
          title="Info"
          value={stats.info}
          icon={<Info className="w-6 h-6" />}
          color="info"
        />
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Bar Chart */}
        <div className="bg-dark-800 rounded-lg border border-dark-700 p-6">
          <h3 className="text-lg font-semibold text-white mb-4">漏洞分布統計</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={barData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#30363d" />
              <XAxis dataKey="name" stroke="#8b949e" />
              <YAxis stroke="#8b949e" />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#161b22',
                  border: '1px solid #30363d',
                  borderRadius: '8px',
                }}
              />
              <Bar dataKey="count" fill="#ef4444" radius={[8, 8, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Pie Chart */}
        <div className="bg-dark-800 rounded-lg border border-dark-700 p-6">
          <h3 className="text-lg font-semibold text-white mb-4">嚴重程度占比</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={severityData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percent }) => `${name}: ${((percent || 0) * 100).toFixed(0)}%`}
                outerRadius={100}
                fill="#8884d8"
                dataKey="value"
              >
                {severityData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip
                contentStyle={{
                  backgroundColor: '#161b22',
                  border: '1px solid #30363d',
                  borderRadius: '8px',
                }}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Recent Findings */}
      <div className="bg-dark-800 rounded-lg border border-dark-700 p-6">
        <h3 className="text-lg font-semibold text-white mb-4">最新發現</h3>
        <div className="space-y-3">
          {mockFindings.map((finding, index) => (
            <div
              key={index}
              className="flex items-start p-4 bg-dark-700 rounded-lg hover:bg-dark-600 transition-colors"
            >
              <div className={`p-2 rounded-lg mr-4 ${getSeverityColor(finding.severity)}`}>
                <AlertTriangle className="w-5 h-5" />
              </div>
              <div className="flex-1">
                <div className="flex items-center justify-between mb-1">
                  <h4 className="font-semibold text-white">{finding.title}</h4>
                  <span className={`text-xs px-2 py-1 rounded ${getSeverityBadge(finding.severity)}`}>
                    {finding.severity}
                  </span>
                </div>
                <p className="text-sm text-dark-300 mb-2">{finding.description}</p>
                <div className="text-xs text-dark-400">
                  {finding.url} • {finding.timestamp}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

interface StatCardProps {
  title: string;
  value: number;
  icon: React.ReactNode;
  color: 'danger' | 'warning' | 'success' | 'info';
  lighter?: boolean;
}

function StatCard({ title, value, icon, color, lighter = false }: StatCardProps) {
  const colorClasses = {
    danger: 'bg-danger-900/20 border-danger-700 text-danger-400',
    warning: lighter ? 'bg-warning-900/15 border-warning-800 text-warning-500' : 'bg-warning-900/20 border-warning-700 text-warning-400',
    success: 'bg-success-900/20 border-success-700 text-success-400',
    info: 'bg-info-900/20 border-info-700 text-info-400',
  };

  return (
    <div className={`rounded-lg border p-4 ${colorClasses[color]}`}>
      <div className="flex items-center justify-between mb-2">
        {icon}
        <span className="text-2xl font-bold">{value}</span>
      </div>
      <div className="text-sm font-medium">{title}</div>
    </div>
  );
}

const mockFindings = [
  {
    severity: 'Critical',
    title: 'SQL Injection 漏洞',
    description: '在登入頁面發現 SQL 注入漏洞，可能導致資料庫洩露',
    url: 'https://wchung.tw/login',
    timestamp: '5 分鐘前',
  },
  {
    severity: 'High',
    title: '缺少安全標頭',
    description: '缺少 Content-Security-Policy 標頭',
    url: 'https://wchung.tw/',
    timestamp: '10 分鐘前',
  },
  {
    severity: 'Medium',
    title: '過時的 JavaScript 庫',
    description: '檢測到使用 jQuery 1.x 版本',
    url: 'https://wchung.tw/',
    timestamp: '15 分鐘前',
  },
];

function getSeverityColor(severity: string) {
  const colors = {
    Critical: 'bg-danger-900/30 text-danger-400',
    High: 'bg-warning-900/30 text-warning-400',
    Medium: 'bg-warning-900/20 text-warning-500',
    Low: 'bg-success-900/30 text-success-400',
    Info: 'bg-info-900/30 text-info-400',
  };
  return colors[severity as keyof typeof colors] || colors.Info;
}

function getSeverityBadge(severity: string) {
  const badges = {
    Critical: 'bg-danger-900/40 text-danger-300 border border-danger-700',
    High: 'bg-warning-900/40 text-warning-300 border border-warning-700',
    Medium: 'bg-warning-900/30 text-warning-400 border border-warning-800',
    Low: 'bg-success-900/40 text-success-300 border border-success-700',
    Info: 'bg-info-900/40 text-info-300 border border-info-700',
  };
  return badges[severity as keyof typeof badges] || badges.Info;
}

export default Dashboard;
