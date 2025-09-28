import { useState } from 'react';
import { User, Bot, Copy, Check, RefreshCw } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface ChatMessageProps {
  message: {
    role: 'user' | 'assistant' | 'system';
    content: string;
    timestamp: number;
  };
}

function ChatMessage({ message }: ChatMessageProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(message.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const isUser = message.role === 'user';
  const isSystem = message.role === 'system';

  if (isSystem) {
    return (
      <div className="flex justify-center py-2 px-6">
        <div className="text-sm text-[var(--text-tertiary)] bg-[var(--bg-secondary)] px-3 py-1 rounded-full">
          {message.content}
        </div>
      </div>
    );
  }

  return (
    <div className={`group py-6 px-4 hover:bg-[var(--hover-bg)] transition-colors message-transition ${
      isUser ? 'bg-transparent' : 'bg-[var(--bg-secondary)]'
    }`}>
      <div className="max-w-4xl mx-auto flex gap-4">
        <div className="flex-shrink-0">
          <div className={`w-8 h-8 rounded-lg flex items-center justify-center ${
            isUser
              ? 'bg-[var(--text-primary)] text-[var(--bg-primary)]'
              : 'bg-gradient-to-br from-blue-500 to-purple-600 text-white'
          }`}>
            {isUser ? <User className="w-5 h-5" /> : <Bot className="w-5 h-5" />}
          </div>
        </div>

        <div className="flex-1 overflow-hidden">
          <div className="flex items-center gap-2 mb-1">
            <span className="font-medium text-sm">
              {isUser ? 'You' : 'BEAR AI'}
            </span>
            <span className="text-xs text-[var(--text-tertiary)]">
              {new Date(message.timestamp).toLocaleTimeString()}
            </span>
          </div>

          <div className="prose prose-sm max-w-none text-[var(--text-primary)] markdown-body">
            {isUser ? (
              <p className="whitespace-pre-wrap">{message.content}</p>
            ) : (
              <ReactMarkdown
                components={{
                  code({ node, inline, className, children, ...props }) {
                    const match = /language-(\w+)/.exec(className || '');
                    return !inline && match ? (
                      <div className="relative group/code">
                        <SyntaxHighlighter
                          language={match[1]}
                          style={oneDark}
                          customStyle={{
                            margin: 0,
                            borderRadius: '6px',
                            fontSize: '13px',
                          }}
                          {...props}
                        >
                          {String(children).replace(/\n$/, '')}
                        </SyntaxHighlighter>
                        <button
                          onClick={() => navigator.clipboard.writeText(String(children))}
                          className="absolute top-2 right-2 p-1.5 bg-gray-700 hover:bg-gray-600 rounded opacity-0 group-hover/code:opacity-100 transition-opacity"
                        >
                          <Copy className="w-3.5 h-3.5 text-white" />
                        </button>
                      </div>
                    ) : (
                      <code className={className} {...props}>
                        {children}
                      </code>
                    );
                  },
                  p: ({ children }) => <p className="mb-3 last:mb-0">{children}</p>,
                  ul: ({ children }) => <ul className="mb-3 last:mb-0">{children}</ul>,
                  ol: ({ children }) => <ol className="mb-3 last:mb-0">{children}</ol>,
                  h1: ({ children }) => <h1 className="text-xl font-semibold mb-3">{children}</h1>,
                  h2: ({ children }) => <h2 className="text-lg font-semibold mb-2">{children}</h2>,
                  h3: ({ children }) => <h3 className="text-base font-semibold mb-2">{children}</h3>,
                }}
              >
                {message.content}
              </ReactMarkdown>
            )}
          </div>

          {!isUser && (
            <div className="flex items-center gap-2 mt-3 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                onClick={handleCopy}
                className="p-1.5 hover:bg-[var(--hover-bg)] rounded transition-all text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
              >
                {copied ? (
                  <Check className="w-4 h-4" />
                ) : (
                  <Copy className="w-4 h-4" />
                )}
              </button>
              <button className="p-1.5 hover:bg-[var(--hover-bg)] rounded transition-all text-[var(--text-tertiary)] hover:text-[var(--text-primary)]">
                <RefreshCw className="w-4 h-4" />
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default ChatMessage;