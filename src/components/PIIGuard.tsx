import React, { useEffect, useState } from 'react';
import { Shield, AlertTriangle } from 'lucide-react';

interface PIIGuardProps {
  text: string;
  onPIIDetected: () => void;
  onPIICleared: () => void;
}

const PIIGuard: React.FC<PIIGuardProps> = ({ text, onPIIDetected, onPIICleared }) => {
  const [detectedPII, setDetectedPII] = useState<string[]>([]);

  useEffect(() => {
    const patterns = [
      { regex: /\b\d{3}-\d{2}-\d{4}\b/, type: 'SSN' },
      { regex: /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/, type: 'Email' },
      { regex: /\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b/, type: 'Phone' },
      { regex: /\b(?:\d{4}[-\s]?){3}\d{4}\b/, type: 'Credit Card' },
      { regex: /\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b/, type: 'IP Address' },
    ];

    const detected: string[] = [];
    patterns.forEach(({ regex, type }) => {
      if (regex.test(text)) {
        detected.push(type);
      }
    });

    setDetectedPII(detected);

    if (detected.length > 0) {
      onPIIDetected();
    } else {
      onPIICleared();
    }
  }, [text, onPIIDetected, onPIICleared]);

  if (detectedPII.length === 0) return null;

  return (
    <div className="absolute bottom-full mb-2 left-0 right-0 bg-red-500/20 border border-red-500 rounded-lg p-3">
      <div className="flex items-start space-x-2">
        <AlertTriangle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
        <div>
          <p className="text-sm text-red-300 font-semibold">PII Detected - Will be automatically removed:</p>
          <div className="flex flex-wrap gap-2 mt-1">
            {detectedPII.map((type) => (
              <span key={type} className="text-xs bg-red-500/30 px-2 py-1 rounded">
                {type}
              </span>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default PIIGuard;