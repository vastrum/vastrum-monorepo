import { useState } from "react";

interface CopyButtonProps {
  value: string;
  label: string;
  className: string;
}

const CopyButton: React.FC<CopyButtonProps> = ({ value, label, className }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      className={`rounded font-medium transition-colors px-5 py-2 text-sm
        ${copied
          ? "bg-green-700 text-white hover:bg-green-600"
          : "bg-gray-700 text-gray-300 hover:text-white hover:bg-gray-600"}
        ${className || ""}`}
      onClick={handleCopy}
    >
      {copied ? "Copied!" : label}
    </button>
  );
};

export default CopyButton;
