declare module 'react-syntax-highlighter' {
  import { ComponentType, ReactNode } from 'react';

  interface SyntaxHighlighterProps {
    language?: string;
    style?: any;
    children?: ReactNode;
    customStyle?: React.CSSProperties;
    codeTagProps?: React.HTMLProps<HTMLElement>;
    useInlineStyles?: boolean;
    showLineNumbers?: boolean;
    showInlineLineNumbers?: boolean;
    startingLineNumber?: number;
    lineNumberContainerStyle?: React.CSSProperties;
    lineNumberStyle?: React.CSSProperties | ((lineNumber: number) => React.CSSProperties);
    wrapLines?: boolean;
    wrapLongLines?: boolean;
    lineProps?: React.HTMLProps<HTMLElement> | ((lineNumber: number) => React.HTMLProps<HTMLElement>);
    renderer?: (props: {
      rows: Array<{
        properties: any;
        children: ReactNode;
      }>;
      stylesheet: any;
      useInlineStyles: boolean;
    }) => ReactNode;
    PreTag?: string | ComponentType<any>;
    CodeTag?: string | ComponentType<any>;
    [spread: string]: any;
  }

  export const Prism: ComponentType<SyntaxHighlighterProps>;
  export default Prism;
}

declare module 'react-syntax-highlighter/dist/esm/styles/prism' {
  export const oneDark: any;
  export const oneLight: any;
  export const prism: any;
  export const tomorrow: any;
  export const twilight: any;
  export const vs: any;
  export const vscDarkPlus: any;
  export const xonokai: any;
}