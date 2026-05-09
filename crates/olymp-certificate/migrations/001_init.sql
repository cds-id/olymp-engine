-- olymp-certificate: certificate_templates, certificates

CREATE TABLE certificate_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id),
    name TEXT NOT NULL,
    template_url TEXT NOT NULL,
    stage_id UUID REFERENCES stages(id),
    -- NULL stage_id = event-level template (participation), non-NULL = stage-specific (winner/qualifier)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES certificate_templates(id),
    participant_stage_id UUID NOT NULL REFERENCES participant_stages(id),
    certificate_url TEXT,
    certificate_number TEXT,
    generated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(template_id, participant_stage_id)
);

CREATE INDEX idx_cert_templates_event ON certificate_templates(event_id);
CREATE INDEX idx_cert_templates_stage ON certificate_templates(stage_id);
CREATE INDEX idx_certificates_participant ON certificates(participant_stage_id);
CREATE INDEX idx_certificates_template ON certificates(template_id);
CREATE INDEX idx_certificates_number ON certificates(certificate_number);
