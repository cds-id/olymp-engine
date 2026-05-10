-- Seed system roles + default permission assignments

-- System roles
INSERT INTO roles (name, description, is_system) VALUES
('superadmin', 'Full system access, manages all events and users', true),
('admin',      'Event administrator, manages specific events',     true),
('panitia',    'Committee member, scoped to stage/region',         true),
('peserta',    'Participant, limited self-service access',         true)
ON CONFLICT (name) DO NOTHING;

-- Superadmin gets ALL permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'superadmin'
ON CONFLICT DO NOTHING;

-- Admin gets everything except RBAC management
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.resource != 'rbac'
ON CONFLICT DO NOTHING;

-- Panitia gets participant/exam/monitoring/ranking view + operational permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'panitia'
  AND p.code IN (
    'participant.view', 'participant.verify', 'participant.approve', 'participant.reject',
    'exam.view', 'exam.monitor',
    'monitoring.view', 'monitoring.flag',
    'ranking.view',
    'region.view'
  )
ON CONFLICT DO NOTHING;

-- Peserta gets minimal self-service
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'peserta'
  AND p.code IN (
    'participant.view',
    'participant.create',
    'exam.view',
    'ranking.view',
    'certificate.view',
    'region.view'
  )
ON CONFLICT DO NOTHING;
