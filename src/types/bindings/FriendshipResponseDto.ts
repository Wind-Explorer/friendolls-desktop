import type { UserBasicDto } from "./UserBasicDto.js";

export type FriendshipResponseDto = {
  id: string;
  friend: UserBasicDto;
  createdAt: string;
};
