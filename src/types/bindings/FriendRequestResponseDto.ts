import type { UserBasicDto } from "./UserBasicDto.js";

export type FriendRequestResponseDto = {
  id: string;
  sender: UserBasicDto;
  receiver: UserBasicDto;
  status: string;
  createdAt: string;
  updatedAt: string;
};
