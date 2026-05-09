-- Seed all permissions (from PRD Section 5)

INSERT INTO permissions (code, resource, action, description) VALUES
-- Master data
('olympiad.master.create',   'olympiad', 'master.create',   'Create events, education levels, subjects'),
('olympiad.master.update',   'olympiad', 'master.update',   'Update events and master data'),
('olympiad.master.delete',   'olympiad', 'master.delete',   'Delete events and master data'),
('olympiad.stage.manage',    'olympiad', 'stage.manage',    'Create/update/transition stages'),

-- Region
('region.view',              'region',   'view',            'View provinces and districts'),
('region.manage',            'region',   'manage',          'Create/update provinces and districts'),

-- Participant
('participant.view',         'participant', 'view',         'View participants'),
('participant.create',       'participant', 'create',       'Register participants'),
('participant.update',       'participant', 'update',       'Update participant data'),
('participant.import',       'participant', 'import',       'Bulk import participants'),
('participant.verify',       'participant', 'verify',       'Verify participant documents'),
('participant.approve',      'participant', 'approve',      'Approve participants'),
('participant.reject',       'participant', 'reject',       'Reject participants'),

-- Exam
('exam.view',                'exam',     'view',            'View exams and questions'),
('exam.create',              'exam',     'create',          'Create exams'),
('exam.update',              'exam',     'update',          'Update exams and questions'),
('exam.assign',              'exam',     'assign',          'Assign participants to exam sessions'),
('exam.monitor',             'exam',     'monitor',         'Monitor live exam sessions'),

-- Ranking
('ranking.view',             'ranking',  'view',            'View rankings'),
('ranking.approve',          'ranking',  'approve',         'Approve/publish rankings'),
('ranking.promote',          'ranking',  'promote',         'Promote qualified participants to next stage'),

-- Monitoring
('monitoring.view',          'monitoring', 'view',          'View exam monitoring dashboard'),
('monitoring.flag',          'monitoring', 'flag',          'Flag suspicious activity'),

-- RBAC
('rbac.role.view',           'rbac',     'role.view',       'View roles and permissions'),
('rbac.role.create',         'rbac',     'role.create',     'Create roles'),
('rbac.role.update',         'rbac',     'role.update',     'Update roles and permissions'),
('rbac.permission.assign',   'rbac',     'permission.assign', 'Assign permissions to roles'),
('rbac.user.assign',         'rbac',     'user.assign',     'Assign roles to users'),
('rbac.audit.view',          'rbac',     'audit.view',      'View RBAC audit logs'),

-- Certificate
('certificate.generate',     'certificate', 'generate',    'Generate certificates'),
('certificate.view',         'certificate', 'view',        'View certificates')
ON CONFLICT (code) DO NOTHING;
