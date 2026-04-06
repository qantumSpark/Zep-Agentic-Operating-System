/* React 19 JSX transform */

export function StatusIndicator({ status }: { status: string }) {
  if (status === "running") {
    return (
      <span className="inline-block w-3 h-3 rounded-full bg-blue-500 animate-pulse" />
    );
  }
  if (status === "success") return <span>✅</span>;
  if (status === "error") return <span>❌</span>;
  return <span>⏳</span>;
}
