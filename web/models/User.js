import mongoose from 'mongoose';

const UserSchema = new mongoose.Schema({
  name: {
    type: String,
    required: true,
  },
  email: {
    type: String,
    required: true,
    unique: true,
  },
  image: String,
  provider: String,
  providerId: String,
}, {
  timestamps: true,
});

export default mongoose.models.User || mongoose.model('User', UserSchema);
