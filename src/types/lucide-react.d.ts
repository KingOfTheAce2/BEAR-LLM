declare module 'lucide-react' {
  import { ComponentType, SVGProps } from 'react';

  interface IconProps extends SVGProps<SVGSVGElement> {
    size?: number | string;
    color?: string;
    strokeWidth?: number;
  }

  export const ChevronDown: ComponentType<IconProps>;
  export const ChevronRight: ComponentType<IconProps>;
  export const ChevronLeft: ComponentType<IconProps>;
  export const ChevronUp: ComponentType<IconProps>;
  export const Plus: ComponentType<IconProps>;
  export const Send: ComponentType<IconProps>;
  export const Upload: ComponentType<IconProps>;
  export const FileText: ComponentType<IconProps>;
  export const Settings: ComponentType<IconProps>;
  export const User: ComponentType<IconProps>;
  export const Bot: ComponentType<IconProps>;
  export const Copy: ComponentType<IconProps>;
  export const RefreshCw: ComponentType<IconProps>;
  export const Check: ComponentType<IconProps>;
  export const AlertCircle: ComponentType<IconProps>;
  export const MessageSquare: ComponentType<IconProps>;
  export const Trash2: ComponentType<IconProps>;
  export const Download: ComponentType<IconProps>;
  export const Shield: ComponentType<IconProps>;
  export const AlertTriangle: ComponentType<IconProps>;
  export const Menu: ComponentType<IconProps>;
  export const X: ComponentType<IconProps>;
  export const Zap: ComponentType<IconProps>;
  export const Wifi: ComponentType<IconProps>;
  export const WifiOff: ComponentType<IconProps>;
  export const Activity: ComponentType<IconProps>;
  export const Clock: ComponentType<IconProps>;
  export const Sun: ComponentType<IconProps>;
  export const Moon: ComponentType<IconProps>;
  export const Paperclip: ComponentType<IconProps>;
  export const Loader2: ComponentType<IconProps>;
  export const Sparkles: ComponentType<IconProps>;
  export const CheckCircle: ComponentType<IconProps>;
  export const XCircle: ComponentType<IconProps>;
  export const Bug: ComponentType<IconProps>;
}