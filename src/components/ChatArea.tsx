import React, { useEffect, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { User, Bot, AlertCircle, Loader2 } from 'lucide-react';

interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

interface ChatAreaProps {
  messages: Message[];
  isLoading: boolean;
}

const ChatArea: React.FC<ChatAreaProps> = ({ messages, isLoading }) => {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  const renderMessage = (message: Message, index: number) => {
    const isUser = message.role === 'user';
    const isSystem = message.role === 'system';

    return (
      <div
        key={index}
        className={`flex ${isUser ? 'justify-end' : 'justify-start'} mb-4`}
      >
        <div
          className={`max-w-3xl flex ${
            isUser ? 'flex-row-reverse' : 'flex-row'
          } items-start space-x-3`}
        >
          <div
            className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center ${
              isUser
                ? 'bg-legal-accent'
                : isSystem
                ? 'bg-yellow-600'
                : 'bg-legal-secondary'
            }`}
          >
            {isUser ? (
              <User className="w-5 h-5" />
            ) : isSystem ? (
              <AlertCircle className="w-5 h-5" />
            ) : (
              <Bot className="w-5 h-5" />
            )}
          </div>

          <div
            className={`px-4 py-2 rounded-lg ${
              isUser
                ? 'bg-legal-accent text-white'
                : isSystem
                ? 'bg-yellow-600/20 text-yellow-200'
                : 'bg-legal-secondary text-gray-100'
            }`}
          >
            {isSystem ? (
              <div className="text-sm">{message.content}</div>
            ) : (
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={{
                  code({ node, inline, className, children, ...props }: any) {
                    const match = /language-(\w+)/.exec(className || '');
                    return !inline && match ? (
                      <SyntaxHighlighter
                        style={vscDarkPlus}
                        language={match[1]}
                        PreTag="div"
                        {...props}
                      >
                        {String(children).replace(/\n$/, '')}
                      </SyntaxHighlighter>
                    ) : (
                      <code className={className} {...props}>
                        {children}
                      </code>
                    );
                  },
                }}
              >
                {message.content}
              </ReactMarkdown>
            )}

            <div className="text-xs opacity-60 mt-2">
              {new Date(message.timestamp).toLocaleTimeString()}
            </div>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div
      ref={scrollRef}
      className="flex-1 overflow-y-auto p-6 space-y-4 scrollbar-thin"
    >
      {messages.length === 0 && (
        <div className="flex flex-col items-center justify-center h-full text-gray-400">
          <Bot className="w-16 h-16 mb-4" />
          <h2 className="text-xl font-semibold mb-2">Legal AI Assistant Ready</h2>
          <p className="text-center max-w-md">
            Ask any legal question or upload documents. All data stays local and PII is automatically removed.
          </p>
        </div>
      )}

      {messages.map(renderMessage)}

      {isLoading && (
        <div className="flex justify-start mb-4">
          <div className="flex items-start space-x-3">
            <div className="w-8 h-8 rounded-full bg-legal-secondary flex items-center justify-center">
              <Bot className="w-5 h-5" />
            </div>
            <div className="px-4 py-2 rounded-lg bg-legal-secondary">
              <Loader2 className="w-5 h-5 animate-spin" />
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ChatArea;