import React from 'react';
import { Activity, Cpu, HardDrive, Thermometer, Shield } from 'lucide-react';

interface StatusBarProps {
  systemStatus: any;
}

const StatusBar: React.FC<StatusBarProps> = ({ systemStatus }) => {
  if (!systemStatus) return null;

  const getStatusColor = (value: number, threshold: number) => {
    if (value < threshold * 0.6) return 'text-green-400';
    if (value < threshold * 0.8) return 'text-yellow-400';
    return 'text-red-400';
  };

  return (
    <div className="bg-legal-dark border-t border-legal-secondary px-4 py-2 flex items-center justify-between text-xs">
      <div className="flex items-center space-x-6">
        <div className="flex items-center space-x-2">
          <Cpu className="w-4 h-4" />
          <span className={getStatusColor(systemStatus.cpu_usage, 85)}>
            CPU: {systemStatus.cpu_usage?.toFixed(1)}%
          </span>
        </div>

        <div className="flex items-center space-x-2">
          <HardDrive className="w-4 h-4" />
          <span className={getStatusColor(systemStatus.memory_usage, 90)}>
            RAM: {systemStatus.memory_usage?.toFixed(1)}%
          </span>
        </div>

        {systemStatus.gpu_usage !== null && (
          <div className="flex items-center space-x-2">
            <Activity className="w-4 h-4" />
            <span className={getStatusColor(systemStatus.gpu_usage, 85)}>
              GPU: {systemStatus.gpu_usage?.toFixed(1)}%
            </span>
          </div>
        )}

        {systemStatus.temperature !== null && (
          <div className="flex items-center space-x-2">
            <Thermometer className="w-4 h-4" />
            <span className={getStatusColor(systemStatus.temperature, 80)}>
              {systemStatus.temperature?.toFixed(1)}°C
            </span>
          </div>
        )}
      </div>

      <div className="flex items-center space-x-2">
        <Shield className={`w-4 h-4 ${systemStatus.is_safe ? 'text-green-400' : 'text-red-400'}`} />
        <span className={systemStatus.is_safe ? 'text-green-400' : 'text-red-400'}>
          {systemStatus.is_safe ? 'System Safe' : 'Resource Warning'}
        </span>
      </div>
    </div>
  );
};

export default StatusBar;