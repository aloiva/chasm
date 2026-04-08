export interface SessionSummary {
  id: string;
  source: string;
  title: string | null;
  turn_count: number;
  cwd: string | null;
  branch: string | null;
  created_at: string | null;
  updated_at: string | null;
  first_message: string | null;
  size_bytes: number | null;
  has_checkpoints: boolean;
  exists_on_disk: boolean;
  storage_path: string | null;
  status: string | null;
  extra: Record<string, string>;
}

export interface ConversationTurn {
  turn_index: number;
  user_message: string | null;
  assistant_response: string | null;
  timestamp: string | null;
}

export interface Checkpoint {
  number: number;
  title: string | null;
  overview: string | null;
  after_turn: number | null;
}

export interface SessionDetail {
  summary: SessionSummary;
  turns: ConversationTurn[];
  checkpoints: Checkpoint[];
  files_touched: string[];
}

export interface ResumeAction {
  type: 'SpawnTerminal' | 'OpenApplication' | 'NotSupported';
  command?: string;
  args?: string[];
  reason?: string;
}

export interface SourceInfo {
  name: string;
  display_name: string;
  icon: string;
  available: boolean;
}
