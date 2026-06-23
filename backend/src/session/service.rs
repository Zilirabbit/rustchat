use std::collections::HashSet;

use async_trait::async_trait;

use crate::{
    auth::types::CurrentUser,
    common::error::{AppError, AppResult},
};

use super::{
    dto::{
        AddGroupMemberRequest, AddGroupMemberResponse, CreateGroupSessionRequest,
        CreateGroupSessionResponse, CreatePrivateSessionRequest, CreatePrivateSessionResponse,
        GroupMemberListItem, LeaveGroupSessionResponse, ListGroupMembersResponse,
        MarkSessionReadResponse, RemoveGroupMemberResponse,
    },
    repo::SessionRepository,
};

const MAX_GROUP_NAME_LENGTH: usize = 100;
const MAX_INITIAL_GROUP_MEMBERS: usize = 100;

#[async_trait]
pub trait SessionUseCase: Send + Sync {
    async fn create_private_session(
        &self,
        current_user: &CurrentUser,
        request: CreatePrivateSessionRequest,
    ) -> AppResult<CreatePrivateSessionResponse>;
    async fn create_group_session(
        &self,
        current_user: &CurrentUser,
        request: CreateGroupSessionRequest,
    ) -> AppResult<CreateGroupSessionResponse>;
    async fn add_group_member(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
        request: AddGroupMemberRequest,
    ) -> AppResult<AddGroupMemberResponse>;
    async fn list_group_members(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
    ) -> AppResult<ListGroupMembersResponse>;
    async fn leave_group_session(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
    ) -> AppResult<LeaveGroupSessionResponse>;
    async fn remove_group_member(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
        user_id: i64,
    ) -> AppResult<RemoveGroupMemberResponse>;
    async fn mark_session_read(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
    ) -> AppResult<MarkSessionReadResponse>;
}

pub struct SessionService<R> {
    repo: R,
}

impl<R> SessionService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> SessionUseCase for SessionService<R>
where
    R: SessionRepository,
{
    async fn create_private_session(
        &self,
        current_user: &CurrentUser,
        request: CreatePrivateSessionRequest,
    ) -> AppResult<CreatePrivateSessionResponse> {
        if request.target_user_id == current_user.user_id {
            return Err(AppError::BadRequest(
                "cannot create private session with yourself".to_string(),
            ));
        }

        if request.target_user_id <= 0 {
            return Err(AppError::BadRequest(
                "target user id must be a positive integer".to_string(),
            ));
        }

        if !self.repo.user_exists(request.target_user_id).await? {
            return Err(AppError::BadRequest(
                "target user does not exist".to_string(),
            ));
        }

        if let Some(session) = self
            .repo
            .find_private_session_between(current_user.user_id, request.target_user_id)
            .await?
        {
            return Ok(CreatePrivateSessionResponse {
                session_id: session.session_id,
                session_type: "private",
                peer_user_id: session.peer_user_id,
                created_at: session.created_at,
                created: false,
            });
        }

        let created_session = self
            .repo
            .create_private_session(current_user.user_id, request.target_user_id)
            .await?;
        let session = created_session.session;

        Ok(CreatePrivateSessionResponse {
            session_id: session.session_id,
            session_type: "private",
            peer_user_id: session.peer_user_id,
            created_at: session.created_at,
            created: created_session.created,
        })
    }

    async fn create_group_session(
        &self,
        current_user: &CurrentUser,
        request: CreateGroupSessionRequest,
    ) -> AppResult<CreateGroupSessionResponse> {
        let name = normalize_group_name(&request.name)?;
        let member_user_ids =
            normalize_group_member_ids(current_user.user_id, request.member_user_ids)?;

        for member_user_id in &member_user_ids {
            if !self.repo.user_exists(*member_user_id).await? {
                return Err(AppError::BadRequest(format!(
                    "user {member_user_id} does not exist"
                )));
            }
        }

        let session = self
            .repo
            .create_group_session(current_user.user_id, &name, &member_user_ids)
            .await?;

        Ok(CreateGroupSessionResponse {
            session_id: session.session_id,
            session_type: "group",
            name: session.name,
            created_by: session.created_by,
            member_user_ids: session.member_user_ids,
            created_at: session.created_at,
        })
    }

    async fn add_group_member(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
        request: AddGroupMemberRequest,
    ) -> AppResult<AddGroupMemberResponse> {
        validate_positive_session_id(session_id)?;

        if request.user_id <= 0 {
            return Err(AppError::BadRequest(
                "user id must be a positive integer".to_string(),
            ));
        }

        let current_member = self
            .repo
            .get_group_member(session_id, current_user.user_id)
            .await?
            .ok_or_else(|| {
                AppError::Forbidden("you are not a member of this group session".to_string())
            })?;

        if current_member.role != "owner" {
            return Err(AppError::Forbidden(
                "only group owner can add members".to_string(),
            ));
        }

        if !self.repo.user_exists(request.user_id).await? {
            return Err(AppError::BadRequest("user does not exist".to_string()));
        }

        if let Some(member) = self
            .repo
            .get_group_member(session_id, request.user_id)
            .await?
        {
            return Ok(AddGroupMemberResponse {
                session_id: member.session_id,
                user_id: member.user_id,
                role: member.role,
                joined_at: member.joined_at,
                added: false,
            });
        }

        let member = self
            .repo
            .add_group_member(session_id, request.user_id)
            .await?;

        Ok(AddGroupMemberResponse {
            session_id: member.session_id,
            user_id: member.user_id,
            role: member.role,
            joined_at: member.joined_at,
            added: true,
        })
    }

    async fn list_group_members(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
    ) -> AppResult<ListGroupMembersResponse> {
        validate_positive_session_id(session_id)?;

        if self
            .repo
            .get_group_member(session_id, current_user.user_id)
            .await?
            .is_none()
        {
            return Err(AppError::Forbidden(
                "you are not a member of this group session".to_string(),
            ));
        }

        let members = self.repo.list_group_members(session_id).await?;

        Ok(ListGroupMembersResponse {
            session_id,
            members: members
                .into_iter()
                .map(|member| GroupMemberListItem {
                    user_id: member.user_id,
                    username: member.username,
                    role: member.role,
                    joined_at: member.joined_at,
                })
                .collect(),
        })
    }

    async fn leave_group_session(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
    ) -> AppResult<LeaveGroupSessionResponse> {
        validate_positive_session_id(session_id)?;

        if self
            .repo
            .get_group_member(session_id, current_user.user_id)
            .await?
            .is_none()
        {
            return Err(AppError::Forbidden(
                "you are not a member of this group session".to_string(),
            ));
        }

        let left = self
            .repo
            .leave_group_session(session_id, current_user.user_id)
            .await?;

        Ok(LeaveGroupSessionResponse {
            session_id,
            user_id: current_user.user_id,
            left,
        })
    }

    async fn remove_group_member(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
        user_id: i64,
    ) -> AppResult<RemoveGroupMemberResponse> {
        validate_positive_session_id(session_id)?;

        if user_id <= 0 {
            return Err(AppError::BadRequest(
                "user id must be a positive integer".to_string(),
            ));
        }

        if user_id == current_user.user_id {
            return Err(AppError::BadRequest(
                "use leave group to remove yourself".to_string(),
            ));
        }

        let current_member = self
            .repo
            .get_group_member(session_id, current_user.user_id)
            .await?
            .ok_or_else(|| {
                AppError::Forbidden("you are not a member of this group session".to_string())
            })?;

        if current_member.role != "owner" {
            return Err(AppError::Forbidden(
                "only group owner can remove members".to_string(),
            ));
        }

        let Some(target_member) = self.repo.get_group_member(session_id, user_id).await? else {
            return Ok(RemoveGroupMemberResponse {
                session_id,
                user_id,
                removed: false,
            });
        };

        if target_member.role == "owner" {
            return Err(AppError::BadRequest(
                "cannot remove a group owner".to_string(),
            ));
        }

        let removed = self.repo.remove_group_member(session_id, user_id).await?;

        Ok(RemoveGroupMemberResponse {
            session_id,
            user_id,
            removed,
        })
    }

    async fn mark_session_read(
        &self,
        current_user: &CurrentUser,
        session_id: i64,
    ) -> AppResult<MarkSessionReadResponse> {
        validate_positive_session_id(session_id)?;

        if !self
            .repo
            .is_session_member(session_id, current_user.user_id)
            .await?
        {
            return Err(AppError::Forbidden(
                "you are not a member of this session".to_string(),
            ));
        }

        let last_message_id = self.repo.get_session_last_message_id(session_id).await?;
        let read_state = self
            .repo
            .mark_session_read(session_id, current_user.user_id, last_message_id)
            .await?;

        Ok(MarkSessionReadResponse {
            session_id: read_state.session_id,
            last_read_message_id: read_state.last_read_message_id,
            last_read_at: read_state.last_read_at,
        })
    }
}

fn validate_positive_session_id(session_id: i64) -> AppResult<()> {
    if session_id <= 0 {
        return Err(AppError::BadRequest(
            "session id must be a positive integer".to_string(),
        ));
    }

    Ok(())
}

fn normalize_group_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest(
            "group name cannot be blank".to_string(),
        ));
    }

    if name.chars().count() > MAX_GROUP_NAME_LENGTH {
        return Err(AppError::BadRequest(format!(
            "group name must be at most {MAX_GROUP_NAME_LENGTH} characters"
        )));
    }

    Ok(name.to_string())
}

fn normalize_group_member_ids(
    owner_user_id: i64,
    requested_member_user_ids: Vec<i64>,
) -> AppResult<Vec<i64>> {
    if requested_member_user_ids.len() > MAX_INITIAL_GROUP_MEMBERS {
        return Err(AppError::BadRequest(format!(
            "group can include at most {MAX_INITIAL_GROUP_MEMBERS} initial members"
        )));
    }

    let mut seen = HashSet::new();
    let mut member_user_ids = Vec::with_capacity(requested_member_user_ids.len() + 1);
    seen.insert(owner_user_id);
    member_user_ids.push(owner_user_id);

    for user_id in requested_member_user_ids {
        if user_id <= 0 {
            return Err(AppError::BadRequest(
                "member user ids must be positive integers".to_string(),
            ));
        }

        if seen.insert(user_id) {
            member_user_ids.push(user_id);
        }
    }

    Ok(member_user_ids)
}

#[derive(Default)]
pub struct UnavailableSessionService;

#[async_trait]
impl SessionUseCase for UnavailableSessionService {
    async fn create_private_session(
        &self,
        _current_user: &CurrentUser,
        _request: CreatePrivateSessionRequest,
    ) -> AppResult<CreatePrivateSessionResponse> {
        Err(AppError::DbNotConfigured)
    }

    async fn create_group_session(
        &self,
        _current_user: &CurrentUser,
        _request: CreateGroupSessionRequest,
    ) -> AppResult<CreateGroupSessionResponse> {
        Err(AppError::DbNotConfigured)
    }

    async fn add_group_member(
        &self,
        _current_user: &CurrentUser,
        _session_id: i64,
        _request: AddGroupMemberRequest,
    ) -> AppResult<AddGroupMemberResponse> {
        Err(AppError::DbNotConfigured)
    }

    async fn leave_group_session(
        &self,
        _current_user: &CurrentUser,
        _session_id: i64,
    ) -> AppResult<LeaveGroupSessionResponse> {
        Err(AppError::DbNotConfigured)
    }

    async fn remove_group_member(
        &self,
        _current_user: &CurrentUser,
        _session_id: i64,
        _user_id: i64,
    ) -> AppResult<RemoveGroupMemberResponse> {
        Err(AppError::DbNotConfigured)
    }

    async fn list_group_members(
        &self,
        _current_user: &CurrentUser,
        _session_id: i64,
    ) -> AppResult<ListGroupMembersResponse> {
        Err(AppError::DbNotConfigured)
    }

    async fn mark_session_read(
        &self,
        _current_user: &CurrentUser,
        _session_id: i64,
    ) -> AppResult<MarkSessionReadResponse> {
        Err(AppError::DbNotConfigured)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::session::{
        model::{
            GroupSession, GroupSessionMember, PrivateSession, SessionMember, SessionReadState,
        },
        repo::{CreatePrivateSessionResult, SessionRepository},
    };

    #[derive(Default)]
    struct FakeSessionRepository {
        next_session_id: Mutex<i64>,
        users: Mutex<HashMap<i64, bool>>,
        sessions: Mutex<HashMap<(i64, i64), PrivateSession>>,
        group_sessions: Mutex<HashMap<i64, GroupSession>>,
        group_members: Mutex<HashMap<(i64, i64), SessionMember>>,
        members: Mutex<HashMap<(i64, i64), bool>>,
        last_message_ids: Mutex<HashMap<i64, Option<i64>>>,
        read_states: Mutex<HashMap<(i64, i64), SessionReadState>>,
    }

    impl FakeSessionRepository {
        fn key(user_id: i64, peer_user_id: i64) -> (i64, i64) {
            if user_id < peer_user_id {
                (user_id, peer_user_id)
            } else {
                (peer_user_id, user_id)
            }
        }

        fn insert_group_member(&self, session_id: i64, user_id: i64, role: &str) {
            self.group_members.lock().unwrap().insert(
                (session_id, user_id),
                SessionMember {
                    session_id,
                    user_id,
                    role: role.to_string(),
                    joined_at: "2026-05-10 12:00:00+00".to_string(),
                },
            );
        }
    }

    #[async_trait]
    impl SessionRepository for FakeSessionRepository {
        async fn user_exists(&self, user_id: i64) -> AppResult<bool> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .get(&user_id)
                .copied()
                .unwrap_or(false))
        }

        async fn is_session_member(&self, session_id: i64, user_id: i64) -> AppResult<bool> {
            Ok(self
                .members
                .lock()
                .unwrap()
                .get(&(session_id, user_id))
                .copied()
                .unwrap_or(false))
        }

        async fn get_session_last_message_id(&self, session_id: i64) -> AppResult<Option<i64>> {
            Ok(self
                .last_message_ids
                .lock()
                .unwrap()
                .get(&session_id)
                .copied()
                .flatten())
        }

        async fn find_private_session_between(
            &self,
            user_id: i64,
            peer_user_id: i64,
        ) -> AppResult<Option<PrivateSession>> {
            let session = self
                .sessions
                .lock()
                .unwrap()
                .get(&Self::key(user_id, peer_user_id))
                .cloned()
                .map(|mut session| {
                    session.peer_user_id = peer_user_id;
                    session
                });
            Ok(session)
        }

        async fn create_private_session(
            &self,
            created_by: i64,
            peer_user_id: i64,
        ) -> AppResult<CreatePrivateSessionResult> {
            let mut next_session_id = self.next_session_id.lock().unwrap();
            *next_session_id += 1;

            let session = PrivateSession {
                session_id: *next_session_id,
                created_by,
                peer_user_id,
                created_at: "2026-05-03 12:00:00+00".to_string(),
            };

            self.sessions
                .lock()
                .unwrap()
                .insert(Self::key(created_by, peer_user_id), session.clone());

            Ok(CreatePrivateSessionResult {
                session,
                created: true,
            })
        }

        async fn create_group_session(
            &self,
            created_by: i64,
            name: &str,
            member_user_ids: &[i64],
        ) -> AppResult<GroupSession> {
            let mut next_session_id = self.next_session_id.lock().unwrap();
            *next_session_id += 1;

            let session = GroupSession {
                session_id: *next_session_id,
                name: name.to_string(),
                created_by,
                member_user_ids: member_user_ids.to_vec(),
                created_at: "2026-05-10 12:00:00+00".to_string(),
            };

            for member_user_id in member_user_ids {
                let role = if *member_user_id == created_by {
                    "owner"
                } else {
                    "member"
                };
                self.group_members.lock().unwrap().insert(
                    (session.session_id, *member_user_id),
                    SessionMember {
                        session_id: session.session_id,
                        user_id: *member_user_id,
                        role: role.to_string(),
                        joined_at: "2026-05-10 12:00:00+00".to_string(),
                    },
                );
            }

            self.group_sessions
                .lock()
                .unwrap()
                .insert(session.session_id, session.clone());

            Ok(session)
        }

        async fn get_group_member(
            &self,
            session_id: i64,
            user_id: i64,
        ) -> AppResult<Option<SessionMember>> {
            Ok(self
                .group_members
                .lock()
                .unwrap()
                .get(&(session_id, user_id))
                .cloned())
        }

        async fn add_group_member(
            &self,
            session_id: i64,
            user_id: i64,
        ) -> AppResult<SessionMember> {
            let member = SessionMember {
                session_id,
                user_id,
                role: "member".to_string(),
                joined_at: "2026-05-10 12:00:00+00".to_string(),
            };
            self.group_members
                .lock()
                .unwrap()
                .insert((session_id, user_id), member.clone());
            Ok(member)
        }

        async fn remove_group_member(&self, session_id: i64, user_id: i64) -> AppResult<bool> {
            Ok(self
                .group_members
                .lock()
                .unwrap()
                .remove(&(session_id, user_id))
                .is_some())
        }

        async fn list_group_members(&self, session_id: i64) -> AppResult<Vec<GroupSessionMember>> {
            let mut members = self
                .group_members
                .lock()
                .unwrap()
                .values()
                .filter(|member| member.session_id == session_id)
                .map(|member| GroupSessionMember {
                    user_id: member.user_id,
                    username: format!("user-{}", member.user_id),
                    role: member.role.clone(),
                    joined_at: member.joined_at.clone(),
                })
                .collect::<Vec<_>>();

            members.sort_by(|left, right| {
                let left_role_rank = if left.role == "owner" { 0 } else { 1 };
                let right_role_rank = if right.role == "owner" { 0 } else { 1 };
                left_role_rank
                    .cmp(&right_role_rank)
                    .then_with(|| left.joined_at.cmp(&right.joined_at))
                    .then_with(|| left.user_id.cmp(&right.user_id))
            });

            Ok(members)
        }

        async fn leave_group_session(&self, session_id: i64, user_id: i64) -> AppResult<bool> {
            Ok(self
                .group_members
                .lock()
                .unwrap()
                .remove(&(session_id, user_id))
                .is_some())
        }

        async fn mark_session_read(
            &self,
            session_id: i64,
            user_id: i64,
            last_read_message_id: Option<i64>,
        ) -> AppResult<SessionReadState> {
            let read_state = SessionReadState {
                session_id,
                user_id,
                last_read_message_id,
                last_read_at: "2026-05-07 12:00:00+00".to_string(),
            };

            self.read_states
                .lock()
                .unwrap()
                .insert((session_id, user_id), read_state.clone());

            Ok(read_state)
        }
    }

    fn current_user() -> CurrentUser {
        CurrentUser {
            user_id: 1,
            username: "alice".to_string(),
        }
    }

    fn bob_user() -> CurrentUser {
        CurrentUser {
            user_id: 2,
            username: "bob".to_string(),
        }
    }

    #[tokio::test]
    async fn create_private_session_creates_new_session() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(2, true);
        let service = SessionService::new(repo);

        let response = service
            .create_private_session(
                &current_user(),
                CreatePrivateSessionRequest { target_user_id: 2 },
            )
            .await
            .unwrap();

        assert_eq!(response.session_id, 1);
        assert_eq!(response.peer_user_id, 2);
        assert!(response.created);
    }

    #[tokio::test]
    async fn create_private_session_reuses_existing_session() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(2, true);
        repo.sessions.lock().unwrap().insert(
            FakeSessionRepository::key(1, 2),
            PrivateSession {
                session_id: 9,
                created_by: 1,
                peer_user_id: 2,
                created_at: "2026-05-03 12:00:00+00".to_string(),
            },
        );

        let service = SessionService::new(repo);
        let response = service
            .create_private_session(
                &current_user(),
                CreatePrivateSessionRequest { target_user_id: 2 },
            )
            .await
            .unwrap();

        assert_eq!(response.session_id, 9);
        assert!(!response.created);
    }

    #[tokio::test]
    async fn create_private_session_reuses_existing_session_from_reverse_direction() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(1, true);
        repo.users.lock().unwrap().insert(2, true);
        let service = SessionService::new(repo);

        let first = service
            .create_private_session(
                &current_user(),
                CreatePrivateSessionRequest { target_user_id: 2 },
            )
            .await
            .unwrap();
        let second = service
            .create_private_session(
                &bob_user(),
                CreatePrivateSessionRequest { target_user_id: 1 },
            )
            .await
            .unwrap();

        assert_eq!(second.session_id, first.session_id);
        assert_eq!(second.peer_user_id, 1);
        assert!(first.created);
        assert!(!second.created);
    }

    #[tokio::test]
    async fn create_private_session_rejects_self_chat() {
        let service = SessionService::new(FakeSessionRepository::default());
        let error = service
            .create_private_session(
                &current_user(),
                CreatePrivateSessionRequest { target_user_id: 1 },
            )
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_group_session_creates_owner_and_members() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(1, true);
        repo.users.lock().unwrap().insert(2, true);
        repo.users.lock().unwrap().insert(3, true);
        let service = SessionService::new(repo);

        let response = service
            .create_group_session(
                &current_user(),
                CreateGroupSessionRequest {
                    name: " team ".to_string(),
                    member_user_ids: vec![2, 3, 2],
                },
            )
            .await
            .unwrap();

        assert_eq!(response.session_type, "group");
        assert_eq!(response.name, "team");
        assert_eq!(response.created_by, 1);
        assert_eq!(response.member_user_ids, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn create_group_session_rejects_missing_member() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(1, true);
        let service = SessionService::new(repo);

        let error = service
            .create_group_session(
                &current_user(),
                CreateGroupSessionRequest {
                    name: "team".to_string(),
                    member_user_ids: vec![2],
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn add_group_member_requires_owner() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(2, true);
        repo.group_members.lock().unwrap().insert(
            (12, 1),
            SessionMember {
                session_id: 12,
                user_id: 1,
                role: "member".to_string(),
                joined_at: "2026-05-10 12:00:00+00".to_string(),
            },
        );
        let service = SessionService::new(repo);

        let error = service
            .add_group_member(&current_user(), 12, AddGroupMemberRequest { user_id: 2 })
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn add_group_member_returns_existing_members_idempotently() {
        let repo = FakeSessionRepository::default();
        repo.users.lock().unwrap().insert(2, true);
        repo.insert_group_member(12, 1, "owner");
        repo.insert_group_member(12, 2, "member");
        let service = SessionService::new(repo);

        let response = service
            .add_group_member(&current_user(), 12, AddGroupMemberRequest { user_id: 2 })
            .await
            .unwrap();

        assert_eq!(response.user_id, 2);
        assert!(!response.added);
    }

    #[tokio::test]
    async fn remove_group_member_allows_owner_to_remove_member() {
        let repo = FakeSessionRepository::default();
        repo.insert_group_member(12, 1, "owner");
        repo.insert_group_member(12, 2, "member");
        let service = SessionService::new(repo);

        let response = service
            .remove_group_member(&current_user(), 12, 2)
            .await
            .unwrap();

        assert_eq!(response.session_id, 12);
        assert_eq!(response.user_id, 2);
        assert!(response.removed);
    }

    #[tokio::test]
    async fn remove_group_member_requires_owner() {
        let repo = FakeSessionRepository::default();
        repo.insert_group_member(12, 1, "member");
        repo.insert_group_member(12, 2, "member");
        let service = SessionService::new(repo);

        let error = service
            .remove_group_member(&current_user(), 12, 2)
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn remove_group_member_rejects_removing_owner() {
        let repo = FakeSessionRepository::default();
        repo.insert_group_member(12, 1, "owner");
        repo.insert_group_member(12, 2, "owner");
        let service = SessionService::new(repo);

        let error = service
            .remove_group_member(&current_user(), 12, 2)
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn leave_group_session_removes_current_member() {
        let repo = FakeSessionRepository::default();
        repo.group_members.lock().unwrap().insert(
            (12, 1),
            SessionMember {
                session_id: 12,
                user_id: 1,
                role: "member".to_string(),
                joined_at: "2026-05-10 12:00:00+00".to_string(),
            },
        );
        let service = SessionService::new(repo);

        let response = service
            .leave_group_session(&current_user(), 12)
            .await
            .unwrap();

        assert_eq!(response.session_id, 12);
        assert_eq!(response.user_id, 1);
        assert!(response.left);
    }

    #[tokio::test]
    async fn list_group_members_returns_members_for_current_member() {
        let repo = FakeSessionRepository::default();
        repo.group_members.lock().unwrap().insert(
            (12, 1),
            SessionMember {
                session_id: 12,
                user_id: 1,
                role: "owner".to_string(),
                joined_at: "2026-05-10 12:00:00+00".to_string(),
            },
        );
        repo.group_members.lock().unwrap().insert(
            (12, 2),
            SessionMember {
                session_id: 12,
                user_id: 2,
                role: "member".to_string(),
                joined_at: "2026-05-10 12:01:00+00".to_string(),
            },
        );
        let service = SessionService::new(repo);

        let response = service
            .list_group_members(&current_user(), 12)
            .await
            .unwrap();

        assert_eq!(response.session_id, 12);
        assert_eq!(response.members.len(), 2);
        assert_eq!(response.members[0].user_id, 1);
        assert_eq!(response.members[0].username, "user-1");
        assert_eq!(response.members[0].role, "owner");
        assert_eq!(response.members[1].user_id, 2);
    }

    #[tokio::test]
    async fn list_group_members_rejects_non_member() {
        let service = SessionService::new(FakeSessionRepository::default());

        let error = service
            .list_group_members(&current_user(), 12)
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn mark_session_read_updates_to_last_message() {
        let repo = FakeSessionRepository::default();
        repo.members.lock().unwrap().insert((12, 1), true);
        repo.last_message_ids.lock().unwrap().insert(12, Some(99));
        let service = SessionService::new(repo);

        let response = service
            .mark_session_read(&current_user(), 12)
            .await
            .unwrap();

        assert_eq!(response.session_id, 12);
        assert_eq!(response.last_read_message_id, Some(99));
        assert_eq!(response.last_read_at, "2026-05-07 12:00:00+00");
    }

    #[tokio::test]
    async fn mark_session_read_allows_empty_session() {
        let repo = FakeSessionRepository::default();
        repo.members.lock().unwrap().insert((12, 1), true);
        repo.last_message_ids.lock().unwrap().insert(12, None);
        let service = SessionService::new(repo);

        let response = service
            .mark_session_read(&current_user(), 12)
            .await
            .unwrap();

        assert_eq!(response.session_id, 12);
        assert_eq!(response.last_read_message_id, None);
    }

    #[tokio::test]
    async fn mark_session_read_rejects_non_member() {
        let service = SessionService::new(FakeSessionRepository::default());

        let error = service
            .mark_session_read(&current_user(), 12)
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::FORBIDDEN);
    }
}
