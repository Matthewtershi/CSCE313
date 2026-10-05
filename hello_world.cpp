 #include <iostream>

int main() {
	for (int row = 1; row <= 20; ++row) {
		for (int column = 1; column <= row; ++column) {
			std::cout << 'X';
		}
        for (int column = row + 1; column <= 20; ++column) {
            std::cout << '#';
        }
		std::cout << '\n';
	}

	return 0;
}
