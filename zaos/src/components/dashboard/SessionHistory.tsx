import { useEffect, useMemo, useState } from "react";
import { useSessionStore } from "../../stores/sessionStore";
import type { CliSession } from "../../stores/sessionStore";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function relativeTime(isoString: string | null): string {
  if (!isoString) return "";
  const now = Date.now();
  const then = new Date(isoString).getTime();
  if (isNaN(then)) return "";

  const diffMs = now - then;
  const seconds = Math.floor(diffMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d ago`;
  if (hours > 0) return `${hours}h ago`;
  if (minutes > 0) return `${minutes}m ago`;
  return "just now";
}

function truncate(text: string | null, maxLen: number): string {
  if (!text) return "";
  return text.length > maxLen ? text.slice(0, maxLen) + "..." : text;
}

// ---------------------------------------------------------------------------
// Session card
// ---------------------------------------------------------------------------

interface SessionCardProps {
  session: CliSession;
}

function SessionCard({ session }: SessionCardProps) {
  const label = session.first_prompt ?? session.last_prompt ?? "Untitled session";

  return (
    <div className="px-3 py-2 bg-zinc-800 rounded border border-zinc-700/50 hover:border-zinc-600 transition-colors">
      <p className="text-xs text-zinc-200 leading-snug truncate">
        {truncate(label, 60)}
      </p>
      <div className="flex items-center justify-between mt-1">
        <span className="text-[10px] text-zinc-500">
          {relativeTime(session.last_modified)}
        </span>
        <span className="text-[10px] text-zinc-600 font-mono">
          {truncate(session.session_id, 12)}
        </span>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// SessionHistory
// ---------------------------------------------------------------------------

export function SessionHistory() {
  const sessions = useSessionStore((s) => s.sessions);
  const sessionsLoaded = useSessionStore((s) => s.sessionsLoaded);
  const loadSessions = useSessionStore((s) => s.loadSessions);
  const [sessionFilter, setSessionFilter] = useState("");

  useEffect(() => {
    if (!sessionsLoaded) loadSessions();
  }, [sessionsLoaded, loadSessions]);

  const sortedSessions = useMemo(() =>
    [...sessions].sort((a, b) => {
      const aTime = a.last_modified ? new Date(a.last_modified).getTime() : 0;
      const bTime = b.last_modified ? new Date(b.last_modified).getTime() : 0;
      return bTime - aTime;
    }),
    [sessions]
  );

  const filteredSessions = useMemo(() => {
    const query = sessionFilter.toLowerCase();
    if (!query) return sortedSessions;
    return sortedSessions.filter((s) => {
      const first = (s.first_prompt ?? "").toLowerCase();
      const last = (s.last_prompt ?? "").toLowerCase();
      return first.includes(query) || last.includes(query);
    });
  }, [sortedSessions, sessionFilter]);

  // ---- Loading state ----
  if (!sessionsLoaded) {
    return (
      <div className="flex items-center justify-center py-6 text-zinc-500 text-sm italic">
        Chargement...
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {/* Search input */}
      <input
        type="text"
        placeholder="Rechercher..."
        value={sessionFilter}
        onChange={(e) => setSessionFilter(e.target.value)}
        className="w-full px-3 py-1.5 rounded bg-zinc-800 border border-zinc-700 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-zinc-500 transition-colors"
      />

      {/* Session count */}
      {filteredSessions.length > 0 && (
        <p className="text-[10px] text-zinc-500">
          {filteredSessions.length} session{filteredSessions.length !== 1 ? "s" : ""}
        </p>
      )}

      {/* Sessions list or empty state */}
      {filteredSessions.length === 0 ? (
        <div className="flex items-center justify-center py-6 text-zinc-500 text-sm italic">
          Aucune session trouvée
        </div>
      ) : (
        <div className="flex flex-col gap-1.5 max-h-64 overflow-y-auto">
          {filteredSessions.map((session) => (
            <SessionCard key={session.session_id} session={session} />
          ))}
        </div>
      )}
    </div>
  );
}
