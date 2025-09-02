import mongoose from 'mongoose';

const ActivityLogSchema = new mongoose.Schema({
  userId: {
    type: mongoose.Schema.Types.ObjectId,
    ref: 'User',
    required: true,
  },
  timestamp: {
    type: Date,
    required: true,
  },
  type: {
    type: String,
    enum: ['window', 'browser', 'keyboard', 'mouse'],
    required: true,
  },
  data: {
    windowTitle: String,
    processName: String,
    url: String,
    browserTitle: String,
    browserName: String,
  }
}, {
  timestamps: true,
});

export default mongoose.models.ActivityLog || mongoose.model('ActivityLog', ActivityLogSchema);
