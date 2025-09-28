import React from 'react';
import {
  FileText, Database, Shield, Brain,
  FolderOpen, Settings, Info, Download
} from 'lucide-react';
import { useAppStore } from '../stores/appStore';

const Sidebar: React.FC = () => {
  const { documents } = useAppStore();

  const menuItems = [
    { icon: Brain, label: 'AI Models', count: 3 },
    { icon: FileText, label: 'Documents', count: documents.length },
    { icon: Database, label: 'Knowledge Base', count: 0 },
    { icon: Shield, label: 'PII Protection', active: true },
    { icon: FolderOpen, label: 'Case Files', count: 0 },
  ];

  return (
    <aside className="w-64 bg-legal-primary border-r border-legal-secondary p-4">
      <div className="space-y-2">
        {menuItems.map((item, index) => (
          <button
            key={index}
            className="w-full flex items-center justify-between px-4 py-3 rounded-lg hover:bg-legal-secondary transition-colors text-left"
          >
            <div className="flex items-center space-x-3">
              <item.icon className="w-5 h-5" />
              <span>{item.label}</span>
            </div>
            {item.count !== undefined && item.count > 0 && (
              <span className="bg-legal-accent px-2 py-1 rounded-full text-xs">
                {item.count}
              </span>
            )}
            {item.active && (
              <span className="w-2 h-2 bg-green-400 rounded-full" />
            )}
          </button>
        ))}
      </div>

      <div className="mt-auto pt-4 border-t border-legal-secondary">
        <button className="w-full flex items-center space-x-3 px-4 py-3 rounded-lg hover:bg-legal-secondary transition-colors">
          <Download className="w-5 h-5" />
          <span>Download Models</span>
        </button>
        <button className="w-full flex items-center space-x-3 px-4 py-3 rounded-lg hover:bg-legal-secondary transition-colors">
          <Settings className="w-5 h-5" />
          <span>Settings</span>
        </button>
        <button className="w-full flex items-center space-x-3 px-4 py-3 rounded-lg hover:bg-legal-secondary transition-colors">
          <Info className="w-5 h-5" />
          <span>About</span>
        </button>
      </div>
    </aside>
  );
};

export default Sidebar;