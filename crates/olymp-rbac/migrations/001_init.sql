-- olymp-rbac: RBAC tables

CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL UNIQUE,
    resource TEXT NOT NULL,
    action TEXT NOT NULL,
    description TEXT
);

CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE user_role_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    role_id UUID NOT NULL REFERENCES roles(id),
    event_id UUID REFERENCES events(id),
    stage_id UUID REFERENCES stages(id),
    province_id UUID REFERENCES provinces(id),
    district_id UUID REFERENCES districts(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMPTZ,
    assigned_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE assignment_permission_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES user_role_assignments(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id),
    granted BOOLEAN NOT NULL,
    UNIQUE(assignment_id, permission_id)
);

CREATE INDEX idx_ura_user ON user_role_assignments(user_id);
CREATE INDEX idx_ura_event ON user_role_assignments(event_id);
CREATE INDEX idx_ura_active ON user_role_assignments(is_active, expires_at);
CREATE INDEX idx_permissions_code ON permissions(code);
CREATE INDEX idx_permissions_resource ON permissions(resource);
