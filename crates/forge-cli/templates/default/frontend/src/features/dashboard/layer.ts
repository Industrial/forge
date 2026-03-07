/**
 * Dashboard feature layer.
 *
 * The app uses the single application layer from {@link getApplicationLayer} / {@link buildApplicationLayer}
 * which provides HttpClient, entity/RPC APIs, subscription stream, and dashboard services.
 * This module is kept for compatibility; use the app layer for all dashboard-related effects.
 */
import { Layer } from 'effect'

/** Stub: use getApplicationLayer() for dashboard effects. */
export const DashboardFeatureLayer = () => Layer.empty
