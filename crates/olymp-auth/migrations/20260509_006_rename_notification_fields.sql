-- Rename e-commerce notification fields to LMS-appropriate names
ALTER TABLE auth.notification_preferences RENAME COLUMN order_updates TO event_updates;
ALTER TABLE auth.notification_preferences RENAME COLUMN promotions TO exam_reminders;
ALTER TABLE auth.notification_preferences RENAME COLUMN newsletter TO result_announcements;
