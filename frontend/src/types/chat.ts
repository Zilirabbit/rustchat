export interface UserProfile {
  user_id: number;
  username: string;
  avatar_url?: string | null;
}

export interface AuthPayload {
  token: string;
  user: UserProfile;
}

export interface UserSearchItem {
  user_id: number;
  username: string;
}

export interface CreatePrivateSessionResponse {
  session_id: number;
  session_type: string;
  peer_user_id: number;
  created_at: string;
  created: boolean;
}

export interface CreateGroupSessionResponse {
  session_id: number;
  session_type: string;
  name: string;
  created_by: number;
  member_user_ids: number[];
  created_at: string;
}

export interface AddGroupMemberResponse {
  session_id: number;
  user_id: number;
  role: string;
  joined_at: string;
  added: boolean;
}

export interface GroupMemberListItem {
  user_id: number;
  username: string;
  role: string;
  joined_at: string;
}

export interface ListGroupMembersResponse {
  session_id: number;
  members: GroupMemberListItem[];
}

export interface LeaveGroupSessionResponse {
  session_id: number;
  user_id: number;
  left: boolean;
}

export interface MarkSessionReadResponse {
  session_id: number;
  last_read_message_id: number | null;
  last_read_at: string;
}

export interface ConversationItem {
  session_id: number;
  session_type: string;
  session_name: string;
  last_message: string | null;
  last_message_time: string | null;
  unread_count: number;
}

export type MessageSendStatus = "queued" | "sending" | "sent" | "failed";

export interface MessageListItem {
  message_id: number;
  session_id: number;
  sender_id: number;
  sender_username: string;
  message_type?: string;
  content: string;
  created_at: string;
  client_message_id?: string;
  send_status?: MessageSendStatus;
  send_error?: string;
  file_id?: number | null;
  file_name?: string | null;
  file_size?: number | null;
  file_type?: string | null;
}

export interface MessageListPage {
  session_id: number;
  limit: number;
  before_message_id: number | null;
  next_before_message_id: number | null;
  has_more: boolean;
  messages: MessageListItem[];
}

export interface WsChatMessage {
  message_id: number;
  session_id: number;
  sender_id: number;
  sender_username: string;
  content: string;
  created_at: string;
  message_type?: string;
  file_id?: number | null;
  file_name?: string | null;
  file_size?: number | null;
  file_type?: string | null;
}

export type ServerEvent =
  | {
      type: "connected";
      user_id: number;
      username: string;
      connection_id: number;
    }
  | { type: "pong" }
  | {
      type: "message_sent";
      message: WsChatMessage;
      client_message_id?: string;
    }
  | { type: "receive_message"; message: WsChatMessage }
  | { type: "error"; message: string; client_message_id?: string };
