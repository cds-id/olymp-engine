-- Allow peserta self-registration into events/stages
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name = 'peserta' AND p.code = 'participant.create'
ON CONFLICT DO NOTHING;
